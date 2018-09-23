use super::super::Flag;
use super::Error;

use std::borrow::Borrow;
use std::fmt;
use std::ops::Range;

#[derive(Clone)]
pub enum Item<S, T> {
    Opt(Opt<S>, T),
    Positional(S),
    Error(Error<S>),
}

impl<S, T> fmt::Debug for Item<S, T>
    where S: Borrow<str>,
          T: fmt::Debug {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Item::Opt(ref opt, ref token) =>
                f.debug_tuple("Opt").field(opt).field(token).finish(),
            Item::Positional(ref positional) =>
                f.debug_tuple("Positional").field(&positional.borrow()).finish(),
            Item::Error(ref error) =>
                f.debug_tuple("Error").field(error).finish(),
        }
    }
}

impl<S1, S2, T1, T2> PartialEq<Item<S2, T2>> for Item<S1, T1>
    where S1: Borrow<str>,
          S2: Borrow<str>,
          T1: PartialEq<T2> {

    fn eq(&self, other: &Item<S2, T2>) -> bool {
        use self::Item::*;
        match (self, other) {
            (&Opt(ref o1, ref t1), &Opt(ref o2, ref t2)) =>
                o1 == o2 && t1 == t2,
            (&Positional(ref p1), &Positional(ref p2)) =>
                p1.borrow() == p2.borrow(),
            (&Error(ref e1), &Error(ref e2)) =>
                e1 == e2,
            _ => false,
        }
    }
}

impl<S, T> Eq for Item<S, T> where S: Borrow<str>, T: Eq { }

#[derive(Clone)]
pub struct Opt<S> {
    inner: InnerOpt<S>,
}

#[derive(Clone)]
enum InnerOpt<S> {
    FlagOpt(FlagOpt<S>),
    ParamOpt(ParamOpt<S>),
}

impl<S> Opt<S> where S: Borrow<str> {
    pub fn new(flag: Flag<S>, param: Option<S>) -> Self {
        let inner = match param {
            None => InnerOpt::FlagOpt(FlagOpt::new(flag)),
            Some(param) => InnerOpt::ParamOpt(ParamOpt::new(flag, param)),
        };

        Opt { inner, }
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

    #[cfg(test)]
    pub (super) fn new_short_flag(c: char) -> Self {
        Opt {
            inner: InnerOpt::FlagOpt(FlagOpt{
                flag: Flag::Short(c),
                range: 0 .. 0
            })
        }
    }

    #[cfg(test)]
    pub (super) fn new_long_flag(original: S, range: Range<usize>) -> Self {
        Opt {
            inner: InnerOpt::FlagOpt(FlagOpt {
                flag: Flag::Long(original),
                range,
            })
        }
    }

    #[cfg(test)]
    pub (super) fn new_short_param(c: char, param_original: S, param_range: Range<usize>)
        -> Self {

        Opt {
            inner: InnerOpt::ParamOpt(ParamOpt {
                flag: Flag::Short(c),
                param_original,
                param_range,
                extra_original: None,
            })
        }
    }

    #[cfg(test)]
    pub (super) fn new_long_param(flag_original: Option<S>, flag_range: Range<usize>,
                                  param_original: S, param_range: Range<usize>)
                                  -> Self {

        Opt {
            inner: InnerOpt::ParamOpt(ParamOpt {
                flag: Flag::Long(flag_range),
                param_original,
                param_range,
                extra_original: flag_original,
            })
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
struct FlagOpt<S> {
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
struct ParamOpt<S> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_flag() {
        let opt1: Opt<&str> = Opt::new(Flag::Short('a'), None);
        let opt2: Opt<&str> = Opt::new_short_flag('a');
        assert_eq!( opt1, opt2 );
    }

    #[test]
    fn long_flag() {
        let opt1: Opt<&str> = Opt::new(Flag::Long("all"), None);
        let opt2: Opt<&str> = Opt::new_long_flag("--all", 2..5);
        assert_eq!( opt1, opt2 );
    }

    #[test]
    fn short_param() {
        let opt1: Opt<&str> = Opt::new(Flag::Short('m'), Some("hello"));
        let opt2: Opt<&str> = Opt::new_short_param('m', "-mhello", 2..7);
        assert_eq!( opt1, opt2 );
    }

    #[test]
    fn long_param_separate() {
        let opt1: Opt<&str> = Opt::new(Flag::Long("message"), Some("hello"));
        let opt2: Opt<&str> = Opt::new_long_param(Some("--message"), 2..9, "hello", 0..5);
        assert_eq!( opt1, opt2 );
    }

    #[test]
    fn long_param_shared() {
        let opt1: Opt<&str> = Opt::new(Flag::Long("message"), Some("hello"));
        let opt2: Opt<&str> = Opt::new_long_param(None, 2..9, "--message=hello", 10..15);
        assert_eq!( opt1, opt2 );
    }
}
