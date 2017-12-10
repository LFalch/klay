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
use klc::WinKeyLayout;

mod keylayout;

fn main() {
    let mut args = args().skip(1);
    let arg = args.next().unwrap();


    let file = File::open(&arg).unwrap();

    if arg.ends_with(".klc") {
        let klc = WinKeyLayout::from_reader(file).unwrap();
        if let Some(arg) = args.next() {
            let file = File::create(arg).unwrap();
            klc.write(file).unwrap();
        }
    } else {
        let keylayout = keylayout::parse(file).unwrap();
        if let Some(arg) = args.next() {
            let file = File::create(arg).unwrap();
            keylayout::write(file, keylayout).unwrap();
        }
    }
}
