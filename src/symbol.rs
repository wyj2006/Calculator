use std::{fmt::Display, hash::Hash};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol {
    name: String,
}

impl Symbol {
    pub fn new(name: &str) -> Symbol {
        Symbol {
            name: name.to_string(),
        }
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
