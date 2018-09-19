use super::super::util::split_first_str;
use super::config::{Config, Presence};
use super::flag::Flag;
use self::Presence::*;

use std::borrow::Borrow;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Iter<'a, Cfg, Arg: 'a> {
    config:     Cfg,
    state:      State<'a>,
    args:       &'a [Arg],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item<'a> {
    Opt(Flag<&'a str>, Option<&'a str>),
    Positional(&'a str),
    Error(ErrorKind<'a>),
}

impl<'a> Item<'a> {
    pub fn is_positional(&self) -> bool {
        match *self {
            Item::Positional(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind<'a> {
    UnknownFlag(Flag<&'a str>),
    MissingParam(Flag<&'a str>),
    UnexpectedParam(Flag<&'a str>, &'a str),
}

#[derive(Clone, Debug)]
enum State<'a> {
    Start,
    ShortOpts(&'a str),
    PositionalOnly,
}

impl<'a, Cfg, Arg> Iter<'a, Cfg, Arg> {
    pub fn new(config: Cfg, args: &'a [Arg]) -> Self {
        Iter {
            config,
            state: State::Start,
            args,
        }
    }

    pub fn config_mut(&mut self) -> &mut Cfg {
        &mut self.config
    }
}

impl<'a, Cfg, Arg> Iter<'a, Cfg, Arg>
    where Cfg: Config,
          Arg: Borrow<str> {

    fn parse_long(&mut self, after_hyphens: &'a str) -> Item<'a> {
        if let Some(index) = after_hyphens.find('=') {
            let long  = &after_hyphens[.. index];
            let param = &after_hyphens[index + 1 ..];
            let flag  = Flag::Long(long);
            match self.config.get_long_param(long) {
                None            => Item::Error(ErrorKind::UnknownFlag(flag)),
                Some(Never)     => Item::Error(ErrorKind::UnexpectedParam(flag, param)),
                Some(IfAttached) => Item::Opt(flag, Some(param)),
                Some(Always)    => Item::Opt(flag, Some(param)),
            }
        } else {
            let long = after_hyphens;
            let flag = Flag::Long(long);
            match self.config.get_long_param(long) {
                None            => Item::Error(ErrorKind::UnknownFlag(flag)),
                Some(Never)     => Item::Opt(flag, None),
                Some(IfAttached) => Item::Opt(flag, None),
                Some(Always)    => match self.next_arg() {
                    None           => Item::Error(ErrorKind::MissingParam(flag)),
                    Some(param)    => Item::Opt(flag, Some(param)),
                },
            }
        }
    }

    fn parse_short(&mut self, c: char, rest: &'a str) -> Item<'a> {
        let flag = Flag::Short(c);

        match self.config.get_short_param(c) {
            None => {
                self.state = State::ShortOpts(rest);
                Item::Error(ErrorKind::UnknownFlag(flag))
            },
            Some(Always) => if rest.is_empty() {
                match self.next_arg() {
                    None      => {
                        // self.state was set to State::Start by next_arg.
                        Item::Error(ErrorKind::MissingParam(flag))
                    },
                    Some(arg) => {
                        self.state = State::Start;
                        Item::Opt(flag, Some(arg))
                    },
                }
            } else {
                self.state = State::Start;
                Item::Opt(flag, Some(rest))
            },
            Some(IfAttached) => if rest.is_empty() {
                Item::Opt(flag, None)
            } else {
                self.state = State::Start;
                Item::Opt(flag, Some(rest))
            },
            Some(Never) => {
                self.state = State::ShortOpts(rest);
                Item::Opt(flag, None)
            }
        }
    }

    fn next_arg(&mut self) -> Option<&'a str> {
        if let Some(arg) = self.args.get(0) {
            self.args = &self.args[1 ..];
            Some(arg.borrow())
        } else {
            self.state = State::Start;
            None
        }
    }
}

impl<'a, Cfg, Arg> Iterator for Iter<'a, Cfg, Arg>
    where Cfg: Config,
          Arg: Borrow<str> {

    type Item = Item<'a>;

    fn next(&mut self) -> Option<Item<'a>> {
        loop {
            match self.state {
                State::Start => {
                    match self.next_arg() {
                        None => return None,

                        Some(arg) => match split_first_str(arg) {
                            Some(('-', rest)) => {
                                match split_first_str(rest) {
                                    None => return Some(Item::Positional(arg)),
                                    Some(('-', "")) => self.state = State::PositionalOnly,
                                    Some(('-', long)) => return Some(self.parse_long(long)),
                                    _ => self.state = State::ShortOpts(rest),
                                }
                            }
                            _ => return Some(Item::Positional(arg)),
                        }
                    }
                }

                State::ShortOpts(rest) => {
                    match split_first_str(rest) {
                        None             => self.state = State::Start,
                        Some((c, rrest)) => return Some(self.parse_short(c, rrest)),
                    }
                }

                State::PositionalOnly => return self.next_arg().map(Item::Positional),
            }
        }
    }
}

impl<'a> fmt::Display for Item<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Item::Opt(Flag::Short(c), None)         => write!(f, "-{}", c),
            Item::Opt(Flag::Long(s), None)          => write!(f, "--{}", s),
            Item::Opt(Flag::Short(c), Some(param))  => write!(f, "-{}{}", c, param),
            Item::Opt(Flag::Long(s), Some(param))   => write!(f, "--{}={}", s, param),
            Item::Positional(arg)                   => f.write_str(arg),
            Item::Error(kind)                       => write!(f, "<error: {}>", kind),
        }
    }
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
