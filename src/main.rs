extern crate utf16_ext;
extern crate linked_hash_map;

use std::env::args_os;
use std::fs::File;

mod klc;
use klc::WinKeyLayout;

fn main() {
    let mut args = args_os().skip(1);
    let arg = args.next().unwrap();
    let file = File::open(arg).unwrap();


    if let Some(arg) = args.next() {
        let klc = WinKeyLayout::from_reader(file).unwrap();
        let file = File::create(arg).unwrap();
        klc.write(file).unwrap();
    }
}
