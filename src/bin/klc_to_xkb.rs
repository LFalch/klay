use key_layout::linux::*;
use key_layout::klc::{Key as KlcKey, WinKeyLayout};
use key_layout::convert::win_to_linux;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::fs::File;
use std::env::args;

fn main() {
    for arg in args().skip(1) {
        let mut path = PathBuf::from(arg);
        let file_klc = File::open(&path).unwrap();
        let layout = WinKeyLayout::from_reader(file_klc).unwrap();

        let layout = convert(layout);

        path.set_extension("");
        let out_file = File::create(path).unwrap();
        layout.write(out_file).unwrap();
    }
}

fn load(p: &str) -> Layout {
    let file = File::open(p).unwrap();

    Layout::from_reader(file).unwrap()
}

fn split_to_file_partial(s: &str) -> (&str, &str) {
    s.find('(')
        .map(|i| (&s[..i], &s[i+1..s.len()-1]))
        .unwrap_or_else(|| (s, "basic"))
}

fn default_keys(p: &str) -> BTreeMap<Key, Output> {
    let (path, part) = split_to_file_partial(p);

    let lay = load(path);
    let part = lay.get_partial(part).unwrap();

    let mut base = if let Some(ref inc) = part.include {
        default_keys(inc)
    } else {
        BTreeMap::new()
    };

    for (&key, output) in part.keys.iter() {
        base.insert(key, output.clone());
    }

    base
}

use std::io::{Write, stdin, stdout};

fn char_or_dead(c: char, deads: &[char]) -> Character {
    if deads.contains(&c) {
        println!("Deadkey `{}' detected.", c);
        print!("Please enter x11 deadkey name (leave empty to ignore the deadkey): dead_");
        stdout().flush().unwrap();
        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();
        line = line.trim().to_owned();

        if line.is_empty() {
            Character::Char(c)
        } else {
            Character::Dead(line.into_boxed_str())
        }
    } else {
        Character::Char(c)
    }
}

fn convert_output(win_key: KlcKey, deads: &[char]) -> Output {
    let normal = win_key.normal.unwrap_or('\0');
    let shift = win_key.shift.unwrap_or('\0');
    let altgr = win_key.ctrl_alt.unwrap_or('\0');
    let altgr_shift = win_key.shift_ctrl_alt.unwrap_or('\0');

    Output {
        normal: char_or_dead(normal, deads),
        shift: char_or_dead(shift, deads),
        altgr: char_or_dead(altgr, deads),
        altgr_shift: char_or_dead(altgr_shift, deads),
    }
}

fn convert(win_layout: WinKeyLayout) -> Layout {
    let mut default_partial = PartialXkbSymbols::new("basic".to_owned());

    default_partial.include = Some("dk(basic)".to_owned());
    default_partial.name_group1 = Some(win_layout.name);

    let deads: Vec<_> = win_layout.deadkeys.keys().copied().collect();

    let default_keys = default_keys("dk(basic)");

    for (scan_code, win_key) in win_layout.layout {
        let key_code = win_to_linux(scan_code);
        let mut output = convert_output(win_key, &deads);

        if let Some(def_out) = default_keys.get(&key_code) {
            output |= def_out.clone();
            if def_out == &output {
                continue;
            }
        }

        default_partial.keys.insert(key_code, output);
    }

    Layout {
        default_partial: default_partial,
        partials: Vec::new(),
    }
}