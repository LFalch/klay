use klay::KlayLayout;

use std::fs::File;
use std::io::{Read, Result};
use std::env::args;

fn main() -> Result<()> {
    for arg in args().skip(1) {
        let mut file = File::open(arg).unwrap();
        let mut s = String::new();
        file.read_to_string(&mut s)?;

        let layout = KlayLayout::from_str(&s);

        println!("{:?}", layout);
    }
    Ok(())
}
