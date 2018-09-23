use super::{ErrorKind, Flag};

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item<'a, T> {
    Opt(Flag<&'a str>, Option<&'a str>, T),
    Positional(&'a str),
    Error(ErrorKind<'a>),
}

impl<'a, T> Item<'a, T> {
    pub fn is_positional(&self) -> bool {
        match *self {
            Item::Positional(_) => true,
            _ => false,
        }
    }
}

impl<'a, T> fmt::Display for Item<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Item::Opt(Flag::Short(c), None, _)         => write!(f, "-{}", c),
            Item::Opt(Flag::Long(s), None, _)          => write!(f, "--{}", s),
            Item::Opt(Flag::Short(c), Some(param), _)  => write!(f, "-{}{}", c, param),
            Item::Opt(Flag::Long(s), Some(param), _)   => write!(f, "--{}={}", s, param),
            Item::Positional(arg)                      => f.write_str(arg),
            Item::Error(kind)                          => write!(f, "<error: {}>", kind),
        }
    }
}
