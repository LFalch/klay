extern crate byteorder;
extern crate linked_hash_map;

mod klc;
mod utf16;

fn main() {
    let klc = klc::read_file("danex.klc").unwrap();
    println!("Layout:\n{:#?}", klc);
}
