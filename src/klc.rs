use std::str::Lines;
use linked_hash_map::LinkedHashMap;

type ScanCode = u8;

#[derive(Debug)]
pub struct Key {
    virtual_key: String,
    cap: u8,
    /// Normal
    state0: Option<char>,
    /// Shift
    state1: Option<char>,
    /// Ctrl
    state2: Option<char>,
    /// Ctrl + Alt
    state6: Option<char>,
    /// Shift + Ctrl + Alt
    state7: Option<char>,
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
    let uint = u32::from_str_radix(hex_codepoint, 16);
    match uint {
        Ok(u) => ::std::char::from_u32(u),
        Err(e) => {
            // TODO Handle ascii normally
            unimplemented!()
        }
    }
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

fn parse(lines: Lines) -> Option<WinKeyLayout> {
    let mut cur_table = Table::None;
    let mut ret = WinKeyLayout::default();

    for line in lines {
        let args = line.split('\t').take_while(|a| !a.starts_with("//")).filter(|a| !a.is_empty());
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
            "\u{feff}KBD" => {
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
                    println!("Line: {}", line);
                    println!("Args: {:?}", args);
                    let key = Key {
                        virtual_key: args[1].to_owned(),
                        cap: args[2].parse().ok()?,
                        state0: read_char(args[3]),
                        state1: read_char(args[4]),
                        state2: read_char(args[5]),
                        state6: read_char(args[6]),
                        state7: read_char(args[7]),
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
                Table::None => panic!("Unknown key `{}'", args[0])
            }
        }
    }

    Some(ret)
}

use std::fs::File;
use utf16::ReadShort;

pub fn read_file(file: &str) -> Option<WinKeyLayout> {
    let mut f = File::open(file).unwrap();
    let s: Vec<_> = f.shorts().collect();
    let s = String::from_utf16(&s).unwrap();
    parse(s.lines())
}
