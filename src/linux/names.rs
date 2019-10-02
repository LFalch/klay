use lazy_static::lazy_static;
use std::collections::HashMap;

pub struct BiMap {
    name_char: HashMap<&'static str, char>,
    char_name: HashMap<char, &'static str>,
}

impl BiMap {
    #[inline]
    fn new() -> Self {
        BiMap {
            name_char: HashMap::new(),
            char_name: HashMap::new(),
        }
    }
    #[inline]
    fn insert(&mut self, name: &'static str, c: char) {
        self.name_char.insert(name, c);
        self.char_name.insert(c, name);
    }
    #[inline]
    pub fn get_char(&self, name: &str) -> Option<char> {
        self.name_char.get(&name).copied()
    }
    #[inline]
    pub fn get_name(&self, c: char) -> Option<&'static str> {
        self.char_name.get(&c).copied()
    }
}

lazy_static! {
    pub static ref NAMES: BiMap = {
        let mut hash = include!(concat!(env!("OUT_DIR"), "/keysymdef.rs"));
        hash.insert("NoSymbol", '\0');
        hash.name_char.shrink_to_fit();
        hash.char_name.shrink_to_fit();
        hash
    };
}
