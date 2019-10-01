use std::collections::BTreeMap;
use std::fmt::{self, Display};
use std::convert::TryFrom;

mod names;
pub use names::NAMES;

pub fn char_to_name(c: char) -> String {
    if let Some(name) = NAMES.get_right(c) {
        name.to_owned()
    } else {
        format!("U{:04x}", c as u32)
    }
}

pub fn name_to_char(name: &str) -> Option<char> {
    if let Some(c) = NAMES.get_left(name) {
        Some(c)
    } else if name.starts_with(name) {
        <char>::try_from(<u32>::from_str_radix(&name[1..], 16).ok()?).ok()
    } else {
        None
    }
}

macro_rules! key {
    ($key:ident; $(
        $code:ident,
    )*) => {
        #[repr(u8)]
        #[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
        pub enum $key {
            $(
                $code
            ),*
        }

        impl $key {
            fn from_str(s: &str) -> Option<Self> {
                use self::$key::*;
                match s {
                    $(stringify!($code) => Some($code),)*
                    _ => None,
                }
            }
        }
    };
}

key! {Key;
    AE01,
    AE02,
    AE03,
    AE04,
    AE05,
    AE06,
    AE07,
    AE08,
    AE09,
    AE10,
    AE11,
    AE12,
    AD01,
    AD02,
    AD03,
    AD04,
    AD05,
    AD06,
    AD07,
    AD08,
    AD09,
    AD10,
    AD11,
    AD12,
    AC01,
    AC02,
    AC03,
    AC04,
    AC05,
    AC06,
    AC07,
    AC08,
    AC09,
    AC10,
    AC11,
    TLDE,
    BKSL,
    AB01,
    AB02,
    AB03,
    AB04,
    AB05,
    AB06,
    AB07,
    AB08,
    AB09,
    AB10,
    SPCE,
    // On ISO keyboards
    LSGT,
}

const TEN: usize = 10;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CharOrDead {
    Char(char),
    Dead(Box<str>),
}
impl Display for CharOrDead {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Self::Char(c) => char_to_name(c).fmt(f),
            Self::Dead(s) => {
                let s = format!("dead_{}", s);
                s.fmt(f)
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Character {
    pub normal: CharOrDead,
    pub shift: CharOrDead,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Output {
    pub normal: Character,
    pub altgr: Option<Character>,
}
impl Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Character{ref normal, ref shift} = self.normal;

        write!(f, "{:>10}, {:>10}", normal, shift)?;
        if let Some(Character{ref normal, ref shift}) = self.altgr {
            write!(f, ", {:>12}, {:>12}", normal, shift)?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct PartialXkbSymbols {
    pub name: String,
    pub include: Option<String>,
    pub name_group1: Option<String>,
    pub keys: BTreeMap<Key, Output>
}
impl Display for PartialXkbSymbols {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "partial alphanumeric_keys")?;
        writeln!(fmt, "xkb_symbols \"{}\" {{\n", self.name)?;

        if let Some(ref inc) = self.include {
            writeln!(fmt, "    include \"{}\"\n", inc)?;
        }
        if let Some(ref ng) = self.name_group1 {
            writeln!(fmt, "    name[Group1]=\"{}\";\n", ng)?;
        }

        for (key, out) in self.keys.iter() {
            writeln!(fmt, "    key <{:?}>\t{{ [{}]\t}};", key, out)?;
        }
        write!(fmt, "}};")
    }
}
impl PartialXkbSymbols {
    pub fn new(name: String) -> Self {
        PartialXkbSymbols {
            name,
            include: None,
            name_group1: None,
            keys: BTreeMap::new(),
        }
    }
    fn process_line(&mut self, line: &str) {
        if line.starts_with("key ") {
            let key = Key::from_str(&line[5..9]).unwrap();
            let start_index = line.find('[').unwrap() + 1;
            let end_index = start_index + line[start_index..].find(']').unwrap();

            let mut chars = line[start_index..end_index]
                .split(',')
                .map(|s| {
                    let s = s.trim();
                    let dead = s.starts_with("dead_");
                    if dead {
                        let s = &s[5..];
                        CharOrDead::Dead(s.to_owned().into_boxed_str())
                    } else {
                        CharOrDead::Char(name_to_char(s).unwrap())
                    }
                });

            let normal = Character {
                normal: chars.next().unwrap(),
                shift: chars.next().unwrap(),
            };
            let altgr = if let Some(next) = chars.next() {
                Some(Character{
                    normal: next,
                    shift: chars.next().unwrap(),
                })
            } else {
                None
            };
            
            self.keys.insert(key, Output{normal, altgr});
        } else if line.starts_with("include ") {
            if self.include.is_none() {
                let start_index = line.find('"').unwrap() + 1;
                let end_index = start_index + line[start_index..].find('"').unwrap();
                let inc_spec = line[start_index..end_index].to_owned();

                self.include = Some(inc_spec);
            }
        } else if line.starts_with("name[Group1]") {
            let start_index = line.find('"').unwrap() + 1;
            let end_index = start_index + line[start_index..].find('"').unwrap();
            let inc_spec = line[start_index..end_index].to_owned();

            self.name_group1 = Some(inc_spec);
        } else {
            eprintln!("Unexpected line {:?}", line);
        }
    }
}
#[derive(Debug, Clone)]
pub struct Layout {
    pub default_partial: PartialXkbSymbols,
    pub partials: Vec<PartialXkbSymbols>,
}

use std::io::{Result, BufReader, BufRead, Read, Write};

impl Layout {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let reader = BufReader::new(reader);

        let mut cur_partial = None;
        let mut is_default = None;
        let mut partials = Vec::new();
        let mut default_partial = None;

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            match (is_default, &mut cur_partial) {
                (None, None) => {
                    assert!(line.contains("partial"));
                    let isd = line.starts_with("default");

                    if isd {
                        assert!(default_partial.is_none());
                    }

                    is_default = Some(isd);
                }
                (Some(_), None) => {
                    assert!(line.starts_with("xkb_symbols "));
                    let end_index = line.rfind('"').unwrap();
                    let name = line[13..end_index].to_owned();
                    cur_partial = Some(PartialXkbSymbols::new(name));
                }
                (Some(default), ref mut cur_partial @ Some(_)) if line.starts_with("}") => {
                    let partial = cur_partial.take().unwrap();
                    is_default = None;

                    if default {
                        default_partial = Some(partial);
                    } else {
                        partials.push(partial);
                    }
                }
                (Some(_), Some(ref mut cur_partial)) => {
                    cur_partial.process_line(line);
                }
                (None, Some(_)) => unreachable!(),
            }
        }

        Ok(Layout {
            default_partial: default_partial.unwrap(),
            partials,
        })
    }
    pub fn get_partial(&self, partial: &str) -> Option<&PartialXkbSymbols> {
        if self.default_partial.name == partial {
            Some(&self.default_partial)
        } else if let Some(i) = self.partials.iter().position(|p| partial == p.name) {
            Some(&self.partials[i])
        } else {
            None
        }
    }
    pub fn write<W: Write>(&self, mut writer: W) -> Result<()> {
        let &Layout{ref default_partial, ref partials} = self;

        writeln!(writer, "default  {}\n", default_partial)?;

        for partial in partials {
            writeln!(writer, "{}\n", partial)?;
        }

        Ok(())
    }
}
