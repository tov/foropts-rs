use super::super::util::split_first_str;
use super::{Config, ErrorKind, Flag, Item, Presence};
use self::Presence::*;

use std::borrow::Borrow;

#[derive(Clone, Debug)]
pub struct SliceIter<'a, Cfg, Arg: 'a> {
    config:     Cfg,
    state:      State<'a>,
    args:       &'a [Arg],
}

#[derive(Clone, Debug)]
enum State<'a> {
    Start,
    ShortOpts(&'a str),
    PositionalOnly,
}

impl<'a, Cfg, Arg> SliceIter<'a, Cfg, Arg>
    where Cfg: Config,
          Arg: Borrow<str> {
    
    pub fn new(config: Cfg, args: &'a [Arg]) -> Self {
        SliceIter {
            config,
            state: State::Start,
            args,
        }
    }

    pub fn config_mut(&mut self) -> &mut Cfg {
        &mut self.config
    }

    pub fn with_config<NewCfg>(self, config: NewCfg) -> SliceIter<'a, NewCfg, Arg> {
        SliceIter {
            config,
            state: self.state,
            args:  self.args,
        }
    }
}

impl<'a, Cfg, Arg> SliceIter<'a, Cfg, Arg>
    where Cfg: Config,
          Arg: Borrow<str> {

    fn parse_long(&mut self, after_hyphens: &'a str) -> Item<'a> {
        if let Some(index) = after_hyphens.find('=') {
            let long  = &after_hyphens[.. index];
            let param = &after_hyphens[index + 1 ..];
            let flag  = Flag::Long(long);
            match self.config.get_long_param(long) {
                None             => Item::Error(ErrorKind::UnknownFlag(flag)),
                Some(Never)      => Item::Error(ErrorKind::UnexpectedParam(flag, param)),
                Some(IfAttached) => Item::Opt(flag, Some(param)),
                Some(Always)     => Item::Opt(flag, Some(param)),
            }
        } else {
            let long = after_hyphens;
            let flag = Flag::Long(long);
            match self.config.get_long_param(long) {
                None             => Item::Error(ErrorKind::UnknownFlag(flag)),
                Some(Never)      => Item::Opt(flag, None),
                Some(IfAttached) => Item::Opt(flag, None),
                Some(Always)     => match self.next_arg() {
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
            Some(IfAttached) => {
                self.state = State::Start;
                if rest.is_empty() {
                    Item::Opt(flag, None)
                } else {
                    Item::Opt(flag, Some(rest))
                }
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

impl<'a, Cfg, Arg> Iterator for SliceIter<'a, Cfg, Arg>
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

