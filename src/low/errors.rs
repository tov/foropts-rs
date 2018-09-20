use super::Flag;

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind<'a> {
    UnknownFlag(Flag<&'a str>),
    MissingParam(Flag<&'a str>),
    UnexpectedParam(Flag<&'a str>, &'a str),
}

impl<'a> fmt::Display for ErrorKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::UnknownFlag(flag) =>
                write!(f, "unknown flag: {}", flag),

            ErrorKind::MissingParam(flag) =>
                write!(f, "missing parameter for: {}", flag),

            ErrorKind::UnexpectedParam(flag, param) =>
                write!(f, "unexpected parameter ‘{}’ for: {}", param, flag),
        }
    }
}
