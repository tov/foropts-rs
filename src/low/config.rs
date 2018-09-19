use super::iter::Iter;
use super::flag::Flag;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Presence {
    Always,
    IfAttached,
    Never,
}

impl From<bool> for Presence {
    fn from(b: bool) -> Self {
        if b { Presence::Always } else { Presence::Never }
    }
}

pub trait Config {
    fn get_short_param(&self, short: char) -> Option<Presence>;

    fn get_long_param(&self, long: &str) -> Option<Presence>;

    fn parse_slice<'a, Arg>(self, args: &'a [Arg]) -> Iter<'a, Self, Arg>
        where Self: Sized,
              Arg:  Borrow<str> + 'a {

        Iter::new(self, args)
    }
}

impl<'a, T: Config> Config for &'a T {
    fn get_short_param(&self, short: char) -> Option<Presence> {
        T::get_short_param(self, short)
    }

    fn get_long_param(&self, long: &str) -> Option<Presence> {
        T::get_long_param(self, long)
    }
}

/// The configuration for the argument parser.
#[derive(Clone, Debug)]
pub struct HashConfig<L>
    where L: Eq + Hash + Borrow<str> {

    short_opts: HashMap<char, Presence>,
    long_opts:  HashMap<L, Presence>,
}

impl<L> Config for HashConfig<L>
    where L: Eq + Hash + Borrow<str> {

    fn get_short_param(&self, short: char) -> Option<Presence> {
        self.short_opts.get(&short).cloned()
    }

    fn get_long_param(&self, long: &str) -> Option<Presence> {
        self.long_opts.get(long).cloned()
    }
}

impl<L> HashConfig<L>
    where L: Eq + Hash + Borrow<str> {

    pub fn new() -> Self {
        HashConfig {
            short_opts: HashMap::new(),
            long_opts:  HashMap::new(),
        }
    }

    pub fn with_capacities(shorts: usize, longs: usize) -> Self {
        HashConfig {
            short_opts: HashMap::with_capacity(shorts),
            long_opts:  HashMap::with_capacity(longs),
        }
    }

    pub fn opt<F, P>(mut self, flag: F, param: P) -> Self
    where F: Into<Flag<L>>,
          P: Into<Presence> {

        match flag.into() {
            Flag::Short(short) => { self.short_opts.insert(short, param.into()); }
            Flag::Long(long)   => { self.long_opts.insert(long, param.into()); }
        }

        self
    }

    pub fn short<P>(mut self, flag: char, param: P) -> Self
        where P: Into<Presence> {

        self.short_opts.insert(flag, param.into());
        self
    }

    pub fn long<S, P>(mut self, flag: S, param: P) -> Self
        where L: From<S>,
              P: Into<Presence> {

        self.long_opts.insert(L::from(flag), param.into());
        self
    }

}

#[derive(Clone, Copy, Debug)]
pub struct SliceConfig<'a> {
    pub short_options: &'a [char],
    pub short_flags:   &'a [char],
    pub long_options:  &'a [&'a str],
    pub long_flags:    &'a [&'a str],
}

impl<'a> Config for SliceConfig<'a> {
    fn get_short_param(&self, short: char) -> Option<Presence> {
        if self.short_options.contains(&short) {
            Some(Presence::Never)
        } else if self.short_flags.contains(&short) {
            Some(Presence::Always)
        } else {
            None
        }
    }

    fn get_long_param(&self, long: &str) -> Option<Presence> {
        if self.long_options.contains(&long) {
            Some(Presence::Never)
        } else if self.long_flags.contains(&long) {
            Some(Presence::Always)
        } else {
            None
        }
    }
}
