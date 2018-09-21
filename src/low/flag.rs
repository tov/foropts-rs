use std::borrow::Borrow;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Flag<L> {
    Short(char),
    Long(L),
}

use self::Flag::*;

impl<L> Flag<L> {
    pub fn is_short(&self) -> bool {
        match *self {
            Short(_) => true,
            Long(_) => false,
        }
    }

    pub fn is_long(&self) -> bool {
        !self.is_short()
    }
}

impl<L: Borrow<str>> Flag<L> {
    pub fn is<'a, F>(&self, other: F) -> bool
        where F: Into<Flag<&'a str>> {

        match (self, other.into()) {
            (&Short(c1),    Short(c2)) => c1 == c2,
            (&Long(ref s1), Long(s2))  => s1.borrow() == s2,
            _                          => false,
        }
    }
}

impl<L> From<char> for Flag<L> {
    fn from(short: char) -> Self {
        Short(short)
    }
}

impl<'a> From<&'a str> for Flag<&'a str> {
    fn from(long: &'a str) -> Self {
        Long(long)
    }
}

impl From<String> for Flag<String> {
    fn from(long: String) -> Self {
        Long(long)
    }
}

impl<'b, L: fmt::Display> fmt::Display for Flag<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Short(short)   => write!(f, "-{}", short),
            Long(ref long) => write!(f, "--{}", long),
        }
    }
}
