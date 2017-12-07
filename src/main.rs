extern crate utf16_ext;
extern crate linked_hash_map;

mod klc;

fn main() {
    let klc = klc::read_file("test.klc").unwrap();
    println!("Layout:\n{:#?}", klc);
}
