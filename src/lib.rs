#[macro_use]
extern crate serde_derive;

mod macos {
    pub mod keylayout;
}
mod windows {
    pub mod klc;
}
pub mod linux;

pub use macos::keylayout;
pub use windows::klc;

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