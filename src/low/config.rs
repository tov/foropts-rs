use std::collections::HashMap;

//pub trait LongOption: Eq + Hash + Deref<Target=str> { }
//
//impl<T: Eq + Hash + Deref<Target=str>> LongOption for T { }

/// The configuration for the argument parser.
#[derive(Clone, Debug)]
pub struct Config {
    pub (crate) short_options: HashMap<char, bool>,
    pub (crate) long_options:  HashMap<String, bool>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            short_options: HashMap::new(),
            long_options:  HashMap::new(),
        }
    }

    pub fn short_flag(mut self, flag: char) -> Self {
        self.short_options.insert(flag, false);
        self
    }

    pub fn short_option(mut self, flag: char) -> Self {
        self.short_options.insert(flag, true);
        self
    }

    pub fn long_flag<S: Into<String>>(mut self, flag: S) -> Self {
        self.long_options.insert(flag.into(), false);
        self
    }

    pub fn long_option<S: Into<String>>(mut self, flag: S) -> Self {
        self.long_options.insert(flag.into(), true);
        self
    }
}

