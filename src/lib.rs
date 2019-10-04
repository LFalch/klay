#[cfg(feature = "macos")]
mod macos {
    pub mod keylayout;
}
#[cfg(feature = "windows")]
mod windows {
    pub mod klc;
}
#[cfg(feature = "linux")]
pub mod linux;

#[cfg(feature = "macos")]
pub use macos::keylayout;
#[cfg(feature = "windows")]
pub use windows::klc;

use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt::{self, Display};
use serde::{Serialize, Deserialize};
use toml::ser;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KlayLayout {
    pub metadata: Metadata,
    // TODO: This doesn't work for whatver reason
    pub keymap: BTreeMap<KeyboardKey, Outs>,
    pub special: BTreeMap<Box<str>, Special>,
}

impl KlayLayout {
    pub fn from_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
    pub fn to_string(&self) -> Result<String, ser::Error> {
        let mut s = String::with_capacity(1024);
        self.serialize(ser::Serializer::new(&mut s).pretty_string(true))?;
        Ok(s)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyboardKey {
    TLD,
    E01,
    E02,
    E03,
    E04,
    E05,
    E06,
    E07,
    E08,
    E09,
    E10,
    #[serde(alias = "pls")]
    E11,
    #[serde(alias = "act")]
    E12,
    D01,
    D02,
    D03,
    D04,
    D05,
    D06,
    D07,
    D08,
    D09,
    D10,
    D11,
    D12,
    C01,
    C02,
    C03,
    C04,
    C05,
    C06,
    C07,
    C08,
    C09,
    C10,
    C11,
    BKS,
    LGT,
    B01,
    B02,
    B03,
    B04,
    B05,
    B06,
    B07,
    #[serde(alias = "cma")]
    B08,
    #[serde(alias = "per")]
    B09,
    #[serde(alias = "min")]
    B10,
    SPC,
    KPD,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<Out>", into = "[Out; 4]")]
pub struct Outs {
    normal: Out,
    shift: Out,
    altgr: Out,
    altgr_shift: Out,
}

impl Into<[Out; 4]> for Outs {
    fn into(self) -> [Out; 4] {
        let Outs {normal, shift, altgr, altgr_shift} = self;
        [normal, shift, altgr, altgr_shift]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TooManyOuts;

impl Display for TooManyOuts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "Too many outputs set".fmt(f)
    }
}

impl TryFrom<Vec<Out>> for Outs {
    type Error = TooManyOuts;
    fn try_from(v: Vec<Out>) -> Result<Self, TooManyOuts> {
        if v.len() > 4 {
            Err(TooManyOuts)
        } else {
            let mut vs = v.into_iter();
            Ok(Outs {
                normal: vs.next().unwrap_or_default(),
                shift: vs.next().unwrap_or_default(),
                altgr: vs.next().unwrap_or_default(),
                altgr_shift: vs.next().unwrap_or_default(),
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Out {
    Char(char),
    Special(Box<str>),
}

impl Default for Out {
    fn default() -> Self {
        Out::Char('\0')
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Special {
    Deadkey {
        deadkey: char,
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub short: String,
    #[serde(default)]
    pub locale: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
}

#[cfg(all(feature = "windows", feature = "linux"))]
pub mod convert {
    use crate::linux::Key;
    use crate::klc::ScanCode;

    macro_rules! convert {
        ($win_to_linux:ident, $linux_to_win:ident; $(
            $win_code:expr => $linux_code:ident,
        )*) => {
            pub fn $win_to_linux(k: ScanCode) -> Key {
                match k {
                    $( $win_code => Key::$linux_code, )*
                    b => unimplemented!("{:02x}", b),
                }
            }
            pub fn $linux_to_win(k: Key) -> ScanCode {
                match k {
                    $( Key::$linux_code => $win_code, )*
                }
            }
        };
    }

    convert!{win_to_linux, linux_to_win; 
        0x02 => AE01,
        0x03 => AE02,
        0x04 => AE03,
        0x05 => AE04,
        0x06 => AE05,
        0x07 => AE06,
        0x08 => AE07,
        0x09 => AE08,
        0x0a => AE09,
        0x0b => AE10,
        0x0c => AE11,
        0x0d => AE12,
        0x10 => AD01,
        0x11 => AD02,
        0x12 => AD03,
        0x13 => AD04,
        0x14 => AD05,
        0x15 => AD06,
        0x16 => AD07,
        0x17 => AD08,
        0x18 => AD09,
        0x19 => AD10,
        0x1a => AD11,
        0x1b => AD12,
        0x1e => AC01,
        0x1f => AC02,
        0x20 => AC03,
        0x21 => AC04,
        0x22 => AC05,
        0x23 => AC06,
        0x24 => AC07,
        0x25 => AC08,
        0x26 => AC09,
        0x27 => AC10,
        0x28 => AC11,
        0x29 => TLDE,
        0x2b => BKSL,
        0x2c => AB01,
        0x2d => AB02,
        0x2e => AB03,
        0x2f => AB04,
        0x30 => AB05,
        0x31 => AB06,
        0x32 => AB07,
        0x33 => AB08,
        0x34 => AB09,
        0x35 => AB10,
        0x39 => SPCE,
        0x56 => LSGT,
        0x53 => KPDL,
    }
}