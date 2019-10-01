use std::env;
use std::io::{Result, BufRead, BufReader, Write};
use std::fs::File;
use std::convert::TryFrom;

fn main() -> Result<()> {
    let out_path = env::var("OUT_DIR").unwrap() + "/keysymdef.rs";

    let keysymdef = BufReader::new(File::open("linux-keysymdef")?);
    let mut out = File::create(out_path)?;

    writeln!(out, "{{let mut map = BiMap::new();")?;

    let mut in_comment_block = false;

    for line in keysymdef.lines() {
        let line = line?;
        let line = line.trim();

        if in_comment_block {
            if line.ends_with("*/") {
                in_comment_block = false;
            }

            continue
        } else if line.starts_with("/*") {
            in_comment_block = true;
            continue;
        } else if line.is_empty() || line.starts_with("//") {
            continue;
        }

        let mut elems = line.split_whitespace();

        let name = elems.next().unwrap();
        let codepoint = <u32>::from_str_radix(&elems.next().unwrap()[1..], 16).unwrap();
        let ch = <char>::try_from(codepoint).unwrap();
        writeln!(out, "map.insert({:?}, {:?});", name, ch)?;
    }

    writeln!(out, "map}}")?;

    Ok(())
}