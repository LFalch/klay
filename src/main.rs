extern crate utf16_ext;
extern crate linked_hash_map;

use std::env::args_os;
use std::fs::File;

mod klc;
use klc::WinKeyLayout;

fn main() {
    let arg = args_os().skip(1).next().unwrap();
    let file = File::open(arg).unwrap();

    let klc = WinKeyLayout::from_reader(file).unwrap();

    println!("Layout:\n{:#?}", klc);
}
