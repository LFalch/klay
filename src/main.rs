extern crate utf16_ext;
extern crate linked_hash_map;

use std::env::args_os;

mod klc;

fn main() {
    let arg = args_os().skip(1).next().unwrap();
    let klc = klc::read_file(arg).unwrap();
    println!("Layout:\n{:#?}", klc);
}
