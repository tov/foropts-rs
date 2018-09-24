use super::Opt;
use std::borrow::Borrow;
use std::fmt;

#[derive(Clone)]
pub struct Error<S> {
    pub (super) kind:   ErrorKind,
    pub (super) option: Opt<S>,
}

impl<S> fmt::Debug for Error<S> where S: Borrow<str> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("option", &self.option)
            .finish()
    }
}

impl<S1, S2> PartialEq<Error<S2>> for Error<S1>
    where S1: Borrow<str>,
          S2: Borrow<str> {

    fn eq(&self, other: &Error<S2>) -> bool {
        self.kind == other.kind && self.option == other.option
    }
}

impl<S> Eq for Error<S> where S: Borrow<str> { }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    UnknownFlag,
    MissingParam,
    UnexpectedParam,
}

