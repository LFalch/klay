extern crate utf16_ext;
extern crate linked_hash_map;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_xml_rs;
extern crate xml;

use std::env::args;
use std::fs::File;

mod klc;
pub use klc::WinKeyLayout;

mod keylayout;
pub use keylayout::{KeyLayout, parse};
mod convert;

fn main() {
    let mut args = args().skip(1);
    let arg = args.next().unwrap();
    let arg1 = args.next().unwrap();

    let file = File::open(&arg).unwrap();
    let klc = WinKeyLayout::from_reader(file).unwrap();
    let keylayout = convert::convert(klc, -400);
    let file = File::create(arg1).unwrap();
    keylayout::write(file, keylayout).unwrap();
}
