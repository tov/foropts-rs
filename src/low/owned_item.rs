use super::Flag;

use std::borrow::Borrow;
use std::fmt;
use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OwnedItem<S> where S: Borrow<str> {
    FlagOpt(FlagOpt<S>),
    ParamOpt(ParamOpt<S>),
    Positional(Positional<S>),
    Error(ErrorKind<S>),
}

#[derive(Clone)]
pub struct FlagOpt<S> where S: Borrow<str> {
    flag:  Flag<S>,
    range: Range<usize>,
}

impl<S: Borrow<str>> FlagOpt<S> {
    pub fn new(flag: Flag<S>) -> Self {
        let len = flag.as_ref().map(|s| s.borrow().len()).into_option().unwrap_or(0);
        FlagOpt {
            flag,
            range: 0..len,
        }
    }

    pub fn flag(&self) -> Flag<&str> {
        self.flag.as_ref().map(|s| &s.borrow()[self.range.clone()])
    }
}

impl<S: Borrow<str>, T: Borrow<str>> PartialEq<FlagOpt<T>> for FlagOpt<S> {
    fn eq(&self, other: &FlagOpt<T>) -> bool {
        self.flag() == other.flag()
    }
}

impl<S: Borrow<str>> Eq for FlagOpt<S> {}

impl<S: Borrow<str>> fmt::Debug for FlagOpt<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FlagOpt")
            .field("flag", &self.flag())
            .finish()
    }
}

#[derive(Clone)]
pub struct ParamOpt<S> where S: Borrow<str> {
    flag:           Flag<Range<usize>>,
    param_original: S,
    param_range:    Range<usize>,
    extra_original: Option<S>,
}

impl<S: Borrow<str>> ParamOpt<S> {
    pub fn new(flag: Flag<S>, param: S) -> Self {
        let param_len = param.borrow().len();
        match flag {
            Flag::Short(c) => ParamOpt {
                flag:           Flag::Short(c),
                param_original: param,
                param_range:    0 .. param_len,
                extra_original: None,
            },

            Flag::Long(s) => {
                let flag_len = s.borrow().len();
                ParamOpt {
                    flag:           Flag::Long(0 .. flag_len),
                    param_original: param,
                    param_range:    0 .. param_len,
                    extra_original: Some(s),
                }
            },
        }
    }

    pub fn flag(&self) -> Flag<&str> {
        match self.flag {
            Flag::Short(c)    => Flag::Short(c),
            Flag::Long(ref s) => {
                let original = self.extra_original.as_ref().unwrap_or(&self.param_original);
                Flag::Long(&original.borrow()[s.clone()])
            }
        }
    }

    pub fn param(&self) -> &str {
        &self.param_original.borrow()[self.param_range.clone()]
    }
}

impl<S: Borrow<str>, T: Borrow<str>> PartialEq<ParamOpt<T>> for ParamOpt<S> {
    fn eq(&self, other: &ParamOpt<T>) -> bool {
        self.flag() == other.flag() && self.param() == other.param()
    }
}

impl<S: Borrow<str>> Eq for ParamOpt<S> {}

impl<S: Borrow<str>> fmt::Debug for ParamOpt<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ParamOpt")
            .field("flag", &self.flag())
            .field("param", &self.param())
            .finish()
    }
}

#[derive(Clone)]
pub struct Positional<S> {
    original:       S,
    range:          Range<usize>,
}

impl<S: Borrow<str>> Positional<S> {
    pub fn new(value: S) -> Self {
        let len = value.borrow().len();
        Positional {
            original:   value,
            range:      0 .. len,
        }
    }

    pub fn value(&self) -> &str {
        &self.original.borrow()[self.range.clone()]
    }
}

impl<S: Borrow<str>, T: Borrow<str>> PartialEq<Positional<T>> for Positional<S> {
    fn eq(&self, other: &Positional<T>) -> bool {
        self.value() == other.value()
    }
}

impl<S: Borrow<str>> Eq for Positional<S> {}

impl<S: Borrow<str>> fmt::Debug for Positional<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Positional")
            .field("value", &self.value())
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind<S> where S: Borrow<str> {
    UnknownFlag(FlagOpt<S>),
    MissingParam(FlagOpt<S>),
    UnexpectedParam(ParamOpt<S>),
}

impl<S> OwnedItem<S> where S: Borrow<str> {
//    pub fn is_positional(&self) -> bool {
//        self.flag.is_none()
//    }
//
//    pub fn is_short(&self) -> bool {
//        self.flag.as_ref().map(Flag::is_short).unwrap_or(false)
//    }
//
//    pub fn is_long(&self) -> bool {
//        self.flag.as_ref().map(Flag::is_long).unwrap_or(false)
//    }
//
//    pub fn has_param(&self) -> bool {
//        self.flag.is_some() && self.original.is_some()
//    }

//    pub fn get_positional(&self) -> Option<&str> {
//        if self.is_positional() {
//            Some(self.original.borrow().index(self.range.clone()))
//        } else {
//            None
//        }
//    }
//
//    pub fn get_flag(&self) -> Option<&str> {
//        if self.is_positional() {
//            None
//        } else {
//            Some(self.original.borrow().index(self.range.clone()))
//        }
//    }
//
//    pub fn get_param(&self) -> Option<&str> {
//        if self.is_positional() || self.extra_range.start >= self.extra_range.end {
//            None
//        } else {
//            let original = self.extra_original.as_ref().unwrap_or(&self.original);
//            Some(original.borrow().index(self.extra_range.clone()))
//        }
//    }
}
