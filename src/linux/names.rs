use lazy_static::lazy_static;
use std::collections::HashMap;
use std::hash::Hash;

pub struct BiMap<L, R> {
    left: HashMap<L, R>,
    right: HashMap<R, L>,
}

impl<L: Eq + Hash + Copy, R: Eq + Hash + Copy> BiMap<L, R> {
    #[inline]
    fn new() -> Self {
        BiMap {
            left: HashMap::new(),
            right: HashMap::new(),
        }
    }
    #[inline]
    fn insert(&mut self, l: L, r: R) {
        self.left.insert(l, r);
        self.right.insert(r, l);
    }
    #[inline]
    pub fn get_left(&self, left: L) -> Option<R> {
        self.left.get(&left).copied()
    }
    #[inline]
    pub fn get_right(&self, right: R) -> Option<L> {
        self.right.get(&right).copied()
    }
}

lazy_static! {
    pub static ref NAMES: BiMap<&'static str, char> = {
        let mut hash = include!(concat!(env!("OUT_DIR"), "/keysymdef.rs"));
        hash.insert("NoSymbol", '\0');
        hash.left.shrink_to_fit();
        hash.right.shrink_to_fit();
        hash
    };
}
