use key_layout::linux::Layout;

use std::fs::File;
use std::env::args;

fn main() {
    for arg in args().skip(1) {
        let file = File::open(arg.clone()).unwrap();
        let layout = Layout::from_reader(file).unwrap();

        let out_file = File::create(arg + ".parsed").unwrap();
        layout.write(out_file).unwrap();
    }
}