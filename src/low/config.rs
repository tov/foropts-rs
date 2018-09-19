use super::iter::Iter;
use super::flag::Flag;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

pub trait Config {
    fn short_takes_param(&self, short: char) -> Option<bool>;

    fn long_takes_param(&self, long: &str) -> Option<bool>;

    fn parse_slice<'a, Arg>(self, args: &'a [Arg]) -> Iter<'a, Self, Arg>
        where Self: Sized,
              Arg:  Borrow<str> + 'a {

        Iter::new(self, args)
    }
}

impl<'a, T: Config> Config for &'a T {
    fn short_takes_param(&self, short: char) -> Option<bool> {
        T::short_takes_param(self, short)
    }

    fn long_takes_param(&self, long: &str) -> Option<bool> {
        T::long_takes_param(self, long)
    }
}

/// The configuration for the argument parser.
#[derive(Clone, Debug)]
pub struct HashConfig<L>
    where L: Eq + Hash + Borrow<str> {

    short_opts: HashMap<char, bool>,
    long_opts:  HashMap<L, bool>,
}

impl<L> Config for HashConfig<L>
    where L: Eq + Hash + Borrow<str> {

    fn short_takes_param(&self, short: char) -> Option<bool> {
        self.short_opts.get(&short).cloned()
    }

    fn long_takes_param(&self, long: &str) -> Option<bool> {
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
            long_opts: HashMap::with_capacity(longs),
        }
    }

    pub fn opt<F: Into<Flag<L>>>(mut self, flag: F, takes_param: bool) -> Self {
        match flag.into() {
            Flag::Short(short) => { self.short_opts.insert(short, takes_param); }
            Flag::Long(long)   => { self.long_opts.insert(long, takes_param); }
        }
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
    fn short_takes_param(&self, short: char) -> Option<bool> {
        if self.short_options.contains(&short) {
            Some(true)
        } else if self.short_flags.contains(&short) {
            Some(false)
        } else {
            None
        }
    }

    fn long_takes_param(&self, long: &str) -> Option<bool> {
        if self.long_options.contains(&long) {
            Some(true)
        } else if self.long_flags.contains(&long) {
            Some(false)
        } else {
            None
        }
    }
}
