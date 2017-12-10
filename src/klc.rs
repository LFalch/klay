use std::io::{Read, Write, Result};

use utf16_ext::{AutoWriter, AutoEndianLines, AutoEndianReader};
use linked_hash_map::LinkedHashMap;

type ScanCode = u8;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CapsLockBehaviour {
    Never = 0,
    ShiftOnCaps = 1,
    ShiftOnCapsAlt = 4,
    ShiftOnCapsAlways = 5,
    // Unsupported for now
    // SgCap = "SGCap"
}

impl From<u8> for CapsLockBehaviour {
    fn from(n: u8) -> Self {
        use self::CapsLockBehaviour::*;
        match n {
            0 => Never,
            1 => ShiftOnCaps,
            4 => ShiftOnCapsAlt,
            5 => ShiftOnCapsAlways,
            _ => panic!("No such behaviour")
        }
    }
}

impl ::std::ops::Add for CapsLockBehaviour {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        (self as u8 + rhs as u8).into()
    }
}

#[derive(Debug)]
pub struct Key {
    pub virtual_key: String,
    /// Capslock Behaviour
    pub cap: CapsLockBehaviour,
    /// Normal (shiftstate 0)
    pub normal: Option<char>,
    /// Shift (shiftstate 1)
    pub shift: Option<char>,
    /// Ctrl (shiftstate 2)
    pub ctrl: Option<char>,
    /// Ctrl + Alt (aka. AltGr) (shiftstate 6)
    pub ctrl_alt: Option<char>,
    /// Shift + Ctrl + Alt (aka. Shift+AltGr) (shiftstate 7)
    pub shift_ctrl_alt: Option<char>,
}

#[derive(Debug, Default)]
pub struct WinKeyLayout {
    pub id: String,
    pub name: String,
    pub copyright: String,
    pub company: String,
    pub locale_name: String,
    pub locale_id: String,
    pub version: String,
    pub layout: LinkedHashMap<ScanCode, Key>,
    pub deadkeys: LinkedHashMap<char, LinkedHashMap<char, char>>,
    pub key_names: LinkedHashMap<ScanCode, String>,
    pub key_names_ext: LinkedHashMap<u8, String>,
    pub keynames_dead: LinkedHashMap<char, String>,
    pub description: String,
    pub language_name: String,
}

enum Table {
    None,
    Attributes,
    ShiftState,
    Layout,
    Deadkey(char),
    Keyname,
    KeynameExt,
    KeynameDead,
    Descriptions,
    LanguageNames
}

fn read_char(mut hex_codepoint: &str) -> Option<char> {
    if hex_codepoint == "-1" {
        return None
    }
    if hex_codepoint.ends_with('@') {
        hex_codepoint = &hex_codepoint[..hex_codepoint.len()-1];
    }
    hex_codepoint.parse().ok().or_else(|| {
        u32::from_str_radix(hex_codepoint, 16).ok().and_then(|u| ::std::char::from_u32(u))
    })
}

fn read_hb(hex_byte: &str) -> Option<u8> {
    u8::from_str_radix(hex_byte, 16).ok()
}

fn st(s: &str) -> String {
    if s.starts_with('"') && s.ends_with('"') {
        s[1..s.len()-1].to_owned()
    } else {
        s.to_owned()
    }
}

fn parse<R: Read>(lines: AutoEndianLines<R>) -> Option<WinKeyLayout> {
    let mut cur_table = Table::None;
    let mut ret = WinKeyLayout::default();

    for line in lines {
        let line = line.unwrap();
        let args = line.split('\t')
            .take_while(|a| !a.starts_with("//") && !a.starts_with(";"))
            .filter(|s| !s.is_empty());
        let args: Vec<_> = args.collect();
        if args.is_empty() {
            continue
        }

        match args[0] {
            "ATTRIBUTES" => cur_table = Table::Attributes,
            "SHIFTSTATE" => cur_table = Table::ShiftState,
            "LAYOUT" => cur_table = Table::Layout,
            "DEADKEY" => {
                let c = read_char(args[1])?;
                ret.deadkeys.insert(c, LinkedHashMap::new());
                cur_table = Table::Deadkey(c)
            }
            "KEYNAME" => cur_table = Table::Keyname,
            "KEYNAME_EXT" => cur_table = Table::KeynameExt,
            "KEYNAME_DEAD" => cur_table = Table::KeynameDead,
            "DESCRIPTIONS" => cur_table = Table::Descriptions,
            "LANGUAGENAMES" => cur_table = Table::LanguageNames,
            "KBD" => {
                ret.id = args[1].to_owned();
                ret.name = st(args[2]);
            }
            "COPYRIGHT" => ret.copyright = st(args[1]),
            "COMPANY" => ret.company = st(args[1]),
            "LOCALENAME" => ret.locale_name = st(args[1]),
            "LOCALEID" => ret.locale_id = st(args[1]),
            "VERSION" => ret.version = args[1].to_owned(),
            "ENDKBD" => break,
            _ => match cur_table {
                // HACK Ignore shift states and attributes
                Table::Attributes => (),
                Table::ShiftState => (),
                Table::Layout => {
                    let key = Key {
                        virtual_key: args[1].to_owned(),
                        cap: args[2].parse::<u8>().ok()?.into(),
                        normal: read_char(args[3]),
                        shift: read_char(args[4]),
                        ctrl: read_char(args[5]),
                        ctrl_alt: read_char(args[6]),
                        shift_ctrl_alt: read_char(args[7]),
                    };
                    ret.layout.insert(read_hb(args[0])?, key);
                },
                Table::Deadkey(ref k) => {
                    ret.deadkeys.get_mut(k)?.insert(read_char(args[0])?, (read_char(args[1])?));
                }
                Table::Keyname => {
                    ret.key_names.insert(read_hb(args[0])?, st(args[1]));
                }
                Table::KeynameExt => {
                    ret.key_names_ext.insert(read_hb(args[0])?, st(args[1]));
                }
                Table::KeynameDead => {
                    ret.keynames_dead.insert(read_char(args[0])?, st(args[1]));
                }
                Table::Descriptions => ret.description = args[1].to_owned(),
                Table::LanguageNames => ret.language_name = args[1].to_owned(),
                Table::None => panic!("Unknown key `{}' with {:?}", args[0], &args[1..])
            }
        }
    }

    Some(ret)
}

fn key_arg<W: Write>(wr: &mut AutoWriter<W>, key: &str, arg: &str) -> Result<()> {
    if !arg.is_empty() {
        wr.write_utf16_string(key)?;
        wr.write_utf16_string("\t\"")?;
        wr.write_utf16_string(arg)?;
        wr.write_utf16_string("\"\r\n\r\n")?;
    }
    Ok(())
}
fn quote_multi(s: &str) -> String {
    if s.chars().any(|c| c.is_whitespace()) {
        format!("\"{}\"", s)
    } else {
        s.to_owned()
    }
}

impl WinKeyLayout {
    pub fn from_reader<R: Read>(reader: R) -> Option<Self> {
        let f = AutoEndianReader::new_auto_bom(reader).unwrap();
        parse(f.utf16_lines())
    }
    pub fn write<W: Write>(&self, writer: W) -> Result<()> {
        // To make sure we don't forget anything we destructure it
        // This will make it so compilation fails if `WinKeyLayout` gains new fields
        let &WinKeyLayout{
            ref id,
            ref name,
            ref copyright,
            ref company,
            ref locale_name,
            ref locale_id,
            ref version,
            ref layout,
            ref deadkeys,
            ref key_names,
            ref key_names_ext,
            ref keynames_dead,
            ref description,
            ref language_name,
        } = self;

        let mut wr = AutoWriter::new_little(writer)?;
        wr.write_utf16_string("KBD\t")?;
        wr.write_utf16_string(id)?;
        wr.write_utf16_string("\t\"")?;
        wr.write_utf16_string(name)?;
        wr.write_utf16_string("\"\r\n\r\n")?;

        key_arg(&mut wr, "COPYRIGHT", copyright)?;
        key_arg(&mut wr, "COMPANY", company)?;
        key_arg(&mut wr, "LOCALENAME", locale_name)?;
        key_arg(&mut wr, "LOCALEID", locale_id)?;
        wr.write_utf16_string("VERSION\t")?;
        wr.write_utf16_string(version)?;
        wr.write_utf16_string("\r\n\r\n")?;
        wr.write_utf16_string("SHIFTSTATE\r\n\r\n0\r\n1 // Shift\r\n2 // Ctrl\r\n6 // AltGr\r\n7 // Shift AltGr\r\n\r\n")?;

        wr.write_utf16_string("LAYOUT\t\t;an '@' indicates dead key\r\n\r\n")?;
        let k = |c: Option<char>| {
            if let Some(c) = c {
                format!("{:04x}{}", c as u32, if deadkeys.contains_key(&c){"@"}else{""})
            } else {
                "-1".to_owned()
            }
        };

        for (scancode, key) in layout {
            let &Key{ref virtual_key, cap, normal, shift, ctrl, ctrl_alt, shift_ctrl_alt} = key;
            let s = format!("{:02x}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\r\n",
                scancode, virtual_key, cap as u8, k(normal), k(shift), k(ctrl), k(ctrl_alt), k(shift_ctrl_alt));
            wr.write_utf16_string(&s)?;
        }
        wr.write_utf16_string("\r\n\r\n")?;

        if !deadkeys.is_empty() {
            for (&deadkey, mappings) in deadkeys {
                wr.write_utf16_string(&format!("DEADKEY\t{:04x}\r\n\r\n", deadkey as u32))?;
                for (&a, &b) in mappings {
                    let s = format!("{:04x}\t{:04x}\t// {} -> {}\r\n", a as u32, b as u32, a, b);
                    wr.write_utf16_string(&s)?;
                }
                wr.write_utf16_string("\r\n")?;
            }
            wr.write_utf16_string("\r\n")?;
        }

        wr.write_utf16_string("KEYNAME\r\n\r\n")?;
        for (scancode, name) in key_names {
            wr.write_utf16_string(&format!("{:02x}\t{}\r\n", scancode, quote_multi(name)))?;
        }
        wr.write_utf16_string("\r\n")?;

        wr.write_utf16_string("KEYNAME_EXT\r\n\r\n")?;
        for (scancode, name) in key_names_ext {
            wr.write_utf16_string(&format!("{:02x}\t{}\r\n", scancode, quote_multi(name)))?;
        }
        wr.write_utf16_string("\r\n")?;

        if !keynames_dead.is_empty() {
            wr.write_utf16_string("KEYNAME_DEAD\r\n\r\n")?;
            for (&c, name) in keynames_dead {
                wr.write_utf16_string(&format!("{:04x}\t\"{}\"\r\n", c as u32, name))?;
            }
            wr.write_utf16_string("\r\n")?;
        }

        wr.write_utf16_string("DESCRIPTIONS\r\n\r\n0409\t")?;
        wr.write_utf16_string(description)?;
        wr.write_utf16_string("\r\n\r\nLANGUAGENAMES\r\n\r\n0409\t")?;
        wr.write_utf16_string(language_name)?;
        wr.write_utf16_string("\r\n\r\nENDKBD\r\n")?;
        Ok(())
    }
}
