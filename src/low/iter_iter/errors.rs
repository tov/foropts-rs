use super::Opt;
use std::borrow::Borrow;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind<S> where S: Borrow<str> {
    UnknownFlag(Opt<S>),
    MissingParam(Opt<S>),
    UnexpectedParam(Opt<S>),
}
