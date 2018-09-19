use ::util::split_first_str;
use super::config::Config;

use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct Iter<'a, 'b, Arg>
    where Arg: Deref<Target=str> + 'b {

    config:     &'a Config,
    state:      State<'b>,
    args:       &'b [Arg],
}

#[derive(Clone, Copy, Debug)]
pub enum Item<'b> {
    Flag(Flag<'b>, Option<&'b str>),
    Positional(&'b str),
    Error(ErrorKind<'b>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Flag<'b> {
    Short(char),
    Long(&'b str),
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorKind<'b> {
    UnknownFlag(Flag<'b>),
    MissingParam(Flag<'b>),
    UnexpectedParam(Flag<'b>),
}

#[derive(Clone, Debug)]
enum State<'b> {
    Start,
    ShortOpts(&'b str),
    PositionalOnly,
    Finished,
}

impl<'a, 'b, Arg> Iter<'a, 'b, Arg> where Arg: Deref<Target=str> + 'b {
    pub fn new(config: &'a Config, args: &'b [Arg]) -> Self {
        Iter {
            config,
            state:  State::Start,
            args,
        }
    }

    fn parse_long(&mut self, after_hyphens: &'b str) -> Item<'b> {
        if let Some(index) = after_hyphens.find('=') {
            let long  = &after_hyphens[.. index];
            let param = &after_hyphens[index + 1 ..];
            let flag  = Flag::Long(long);
            match self.long_takes_param(long) {
                None        => Item::Error(ErrorKind::UnknownFlag(flag)),
                Some(false) => Item::Error(ErrorKind::UnexpectedParam(flag)),
                Some(true)  => Item::Flag(flag, Some(param)),
            }
        } else {
            let long = after_hyphens;
            let flag = Flag::Long(long);
            match self.long_takes_param(long) {
                None        => Item::Error(ErrorKind::UnknownFlag(flag)),
                Some(false) => Item::Flag(flag, None),
                Some(true)  => match self.next_arg() {
                    None        => Item::Error(ErrorKind::MissingParam(flag)),
                    Some(param) => Item::Flag(flag, Some(param)),
                },
            }
        }
    }

    fn parse_short(&mut self, c: char, rest: &'b str) -> Item<'b> {
        let flag = Flag::Short(c);

        match self.short_takes_param(c) {
            None => {
                self.state = State::ShortOpts(rest);
                Item::Error(ErrorKind::UnknownFlag(flag))
            },
            Some(true) => if rest.is_empty() {
                match self.next_arg() {
                    None      => {
                        // self.state was set to State::Finished by next_arg.
                        Item::Error(ErrorKind::MissingParam(flag))
                    },
                    Some(arg) => {
                        self.state = State::Start;
                        Item::Flag(flag, Some(arg))
                    },
                }
            } else {
                self.state = State::Start;
                Item::Flag(flag, Some(rest))
            },
            Some(false) => {
                self.state = State::ShortOpts(rest);
                Item::Flag(flag, None)
            }
        }
    }

    fn next_arg(&mut self) -> Option<&'b str> {
        if let Some(arg) = self.args.get(0) {
            self.args = &self.args[1 ..];
            Some(arg)
        } else {
            self.state = State::Finished;
            None
        }
    }

    fn long_takes_param(&self, name: &str) -> Option<bool> {
        self.config.long_options.get(name).cloned()
    }

    fn short_takes_param(&self, name: char) -> Option<bool> {
        self.config.short_options.get(&name).cloned()
    }
}

impl<'a, 'b, Arg> Iterator for Iter<'a, 'b, Arg>
    where Arg: Deref<Target=str> + 'b {

    type Item = Item<'b>;

    fn next(&mut self) -> Option<Item<'b>> {
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

                State::Finished => return None,
            }
        }
    }
}
