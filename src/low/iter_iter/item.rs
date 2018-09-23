use super::super::Flag;
use super::ErrorKind;

use std::borrow::Borrow;
use std::fmt;
use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Item<S> where S: Borrow<str> {
    Opt(Opt<S>),
    Positional(Positional<S>),
    Error(ErrorKind<S>),
}

#[derive(Clone)]
pub struct Opt<S> where S: Borrow<str> {
    inner: InnerOpt<S>,
}

#[derive(Clone)]
enum InnerOpt<S> where S: Borrow<str> {
    FlagOpt(FlagOpt<S>),
    ParamOpt(ParamOpt<S>),
}

impl<S> Opt<S> where S: Borrow<str> {
    pub fn new(flag: Flag<S>, param: Option<S>) -> Self {
        let inner = match param {
            None => InnerOpt::FlagOpt(FlagOpt::new(flag)),
            Some(param) => InnerOpt::ParamOpt(ParamOpt::new(flag, param)),
        };

        Opt { inner }
    }

    pub fn flag(&self) -> Flag<&str> {
        match self.inner {
            InnerOpt::FlagOpt(ref inner) => inner.flag(),
            InnerOpt::ParamOpt(ref inner) => inner.flag(),
        }
    }

    pub fn param(&self) -> Option<&str> {
        match self.inner {
            InnerOpt::FlagOpt(_)          => None,
            InnerOpt::ParamOpt(ref inner) => Some(inner.param()),
        }
    }
}

impl<S, T> PartialEq<Opt<T>> for Opt<S>
    where S: Borrow<str>,
          T: Borrow<str> {

    fn eq(&self, other: &Opt<T>) -> bool {
        self.flag() == other.flag() && self.param() == other.param()
    }
}

impl<S> Eq for Opt<S> where S: Borrow<str> {}

impl<S> fmt::Debug for Opt<S> where S: Borrow<str> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Opt")
            .field("flag", &self.flag())
            .field("param", &self.param())
            .finish()
    }
}

#[derive(Clone)]
struct FlagOpt<S> where S: Borrow<str> {
    flag:  Flag<S>,
    range: Range<usize>,
}

impl<S: Borrow<str>> FlagOpt<S> {
    fn new(flag: Flag<S>) -> Self {
        let len = flag.as_ref().map(|s| s.borrow().len()).into_option().unwrap_or(0);
        FlagOpt {
            flag,
            range: 0..len,
        }
    }

    fn flag(&self) -> Flag<&str> {
        self.flag.as_ref().map(|s| &s.borrow()[self.range.clone()])
    }
}

#[derive(Clone)]
struct ParamOpt<S> where S: Borrow<str> {
    flag:           Flag<Range<usize>>,
    param_original: S,
    param_range:    Range<usize>,
    extra_original: Option<S>,
}

impl<S: Borrow<str>> ParamOpt<S> {
    fn new(flag: Flag<S>, param: S) -> Self {
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

    fn flag(&self) -> Flag<&str> {
        match self.flag {
            Flag::Short(c)    => Flag::Short(c),
            Flag::Long(ref s) => {
                let original = self.extra_original.as_ref().unwrap_or(&self.param_original);
                Flag::Long(&original.borrow()[s.clone()])
            }
        }
    }

    fn param(&self) -> &str {
        &self.param_original.borrow()[self.param_range.clone()]
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
