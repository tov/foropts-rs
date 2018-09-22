use super::super::*;

// Result-building helpers

pub fn flag<'a, F: Into<Flag<&'a str>>>(flag: F) -> Item<'a, ()> {
    Item::Opt(flag.into(), None, ())
}

pub fn opt_with<'a, F: Into<Flag<&'a str>>>(flag: F, param: &'a str) -> Item<'a, ()> {
    Item::Opt(flag.into(), Some(param), ())
}

pub fn pos(value: &str) -> Item<()> {
    Item::Positional(value)
}

pub fn unknown<'a, F: Into<Flag<&'a str>>>(flag: F) -> Item<'a, ()> {
    Item::Error(ErrorKind::UnknownFlag(flag.into()))
}

pub fn unexpected_param<'a, F: Into<Flag<&'a str>>>(flag: F, param: &'a str) -> Item<'a, ()> {
    Item::Error(ErrorKind::UnexpectedParam(flag.into(), param))
}

pub fn missing_param<'a, F: Into<Flag<&'a str>>>(flag: F) -> Item<'a, ()> {
    Item::Error(ErrorKind::MissingParam(flag.into()))
}
