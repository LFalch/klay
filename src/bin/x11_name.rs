use klay::linux::char_to_name;

use std::env::args;
use std::convert::TryFrom;

fn main() {
    for arg in args().skip(1) {
        let c = if arg.starts_with("U") {
            if let Ok(codepoint) = <u32>::from_str_radix(&arg[1..], 16) {
                <char>::try_from(codepoint).ok()
            } else {
                arg.parse().ok()
            }
        } else { arg.parse().ok() };
        
        if let Some(c) = c {
            println!("{}", char_to_name(c));
        } else {
            eprintln!("error: couldn't translate `{}'", arg);
        }
    }
}