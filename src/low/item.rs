use super::{ErrorKind, Flag};

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item<'a> {
    Opt(Flag<&'a str>, Option<&'a str>),
    Positional(&'a str),
    Error(ErrorKind<'a>),
}

impl<'a> Item<'a> {
    pub fn is_positional(&self) -> bool {
        match *self {
            Item::Positional(_) => true,
            _ => false,
        }
    }
}

impl<'a> fmt::Display for Item<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Item::Opt(Flag::Short(c), None)         => write!(f, "-{}", c),
            Item::Opt(Flag::Long(s), None)          => write!(f, "--{}", s),
            Item::Opt(Flag::Short(c), Some(param))  => write!(f, "-{}{}", c, param),
            Item::Opt(Flag::Long(s), Some(param))   => write!(f, "--{}={}", s, param),
            Item::Positional(arg)                   => f.write_str(arg),
            Item::Error(kind)                       => write!(f, "<error: {}>", kind),
        }
    }
}
