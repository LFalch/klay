use std::fs::File;

use utf16_ext::{AutoEndianLines, AutoEndianReader};
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
    virtual_key: String,
    /// Capslock Behaviour
    cap: CapsLockBehaviour,
    /// Normal (shiftstate 0)
    normal: Option<char>,
    /// Shift (shiftstate 1)
    shift: Option<char>,
    /// Ctrl (shiftstate 2)
    ctrl: Option<char>,
    /// Ctrl + Alt (aka. AltGr) (shiftstate 6)
    ctrl_alt: Option<char>,
    /// Shift + Ctrl + Alt (aka. Shift+AltGr) (shiftstate 7)
    shift_ctrl_alt: Option<char>,
}

#[derive(Debug, Default)]
pub struct WinKeyLayout {
    id: String,
    name: String,
    copyright: String,
    company: String,
    locale_name: String,
    locale_id: String,
    version: String,
    layout: LinkedHashMap<ScanCode, Key>,
    deadkeys: LinkedHashMap<char, LinkedHashMap<char, char>>,
    key_names: LinkedHashMap<u8, String>,
    key_names_ext: LinkedHashMap<u8, String>,
    keynames_dead: LinkedHashMap<char, String>,
    description: String,
    language_name: String,
}

enum Table {
    None,
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

fn parse(lines: AutoEndianLines<File>) -> Option<WinKeyLayout> {
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
                // HACK Ignore shift states
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

use std::path::Path;

pub fn read_file<P: AsRef<Path>>(file: P) -> Option<WinKeyLayout> {
    let f = AutoEndianReader::new_auto_bom(File::open(file).unwrap()).unwrap();
    parse(f.utf16_lines())
}
