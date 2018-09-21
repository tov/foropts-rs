use super::super::util::split_first_str;
use super::common::InnerState;
use super::{Config, ErrorKind, Flag, Item, Presence};
use self::Presence::*;

use std::borrow::Borrow;
use std::fmt;

#[derive(Clone)]
pub struct SliceIter<'a, Cfg, Arg: 'a> {
    config:     Cfg,
    state:      State<'a, Arg>,
}

#[derive(Clone)]
struct State<'a, Arg: 'a> {
    first:      InnerState<'a>,
    rest:       &'a [Arg],
}

impl<'a, Cfg, Arg> SliceIter<'a, Cfg, Arg>
    where Cfg: Config,
          Arg: Borrow<str> {

    pub fn new(config: Cfg, args: &'a [Arg]) -> Self {
        SliceIter {
            config,
            state: State {
                first: InnerState::Start,
                rest: args,
            }
        }
    }

    pub fn config_mut(&mut self) -> &mut Cfg {
        &mut self.config
    }

    pub fn next_with_config<Cfg1: Config>(&mut self, config: Cfg1) -> Option<Item<'a>> {
        self.state.next_with_config(config)
    }

    pub fn with_config<Cfg1>(self, config: Cfg1) -> SliceIter<'a, Cfg1, Arg> {
        SliceIter {
            config,
            state: self.state,
        }
    }
}

impl<'a, Arg> State<'a, Arg> where Arg: Borrow<str> {
    pub fn next_with_config<Cfg: Config>(&mut self, config: Cfg) -> Option<Item<'a>> {
        loop {
            match self.first {
                InnerState::Start => {
                    match self.next_arg() {
                        None => return None,

                        Some(arg) => match split_first_str(arg) {
                            Some(('-', rest)) => {
                                match split_first_str(rest) {
                                    None => return Some(Item::Positional(arg)),
                                    Some(('-', "")) =>
                                        self.first = InnerState::PositionalOnly,
                                    Some(('-', long)) =>
                                        return Some(self.parse_long(config, long)),
                                    _ => self.first = InnerState::ShortOpts(rest),
                                }
                            }
                            _ => return Some(Item::Positional(arg)),
                        }
                    }
                }

                InnerState::ShortOpts(rest) => {
                    match split_first_str(rest) {
                        None             => self.first = InnerState::Start,
                        Some((c, rrest)) =>
                            return Some(self.parse_short(config, c, rrest)),
                    }
                }

                InnerState::PositionalOnly => return self.next_arg().map(Item::Positional),
            }
        }
    }

    fn parse_long<Cfg: Config>(&mut self, config: Cfg, after_hyphens: &'a str) -> Item<'a> {
        if let Some(index) = after_hyphens.find('=') {
            let long  = &after_hyphens[.. index];
            let param = &after_hyphens[index + 1 ..];
            let flag  = Flag::Long(long);
            match config.get_long_param(long) {
                None             => Item::Error(ErrorKind::UnknownFlag(flag)),
                Some(Never)      => Item::Error(ErrorKind::UnexpectedParam(flag, param)),
                Some(IfAttached) => Item::Opt(flag, Some(param)),
                Some(Always)     => Item::Opt(flag, Some(param)),
            }
        } else {
            let long = after_hyphens;
            let flag = Flag::Long(long);
            match config.get_long_param(long) {
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

    fn parse_short<Cfg: Config>(&mut self, config: Cfg, c: char, rest: &'a str) -> Item<'a> {
        let flag = Flag::Short(c);

        match config.get_short_param(c) {
            None => {
                self.first = InnerState::ShortOpts(rest);
                Item::Error(ErrorKind::UnknownFlag(flag))
            },
            Some(Always) => if rest.is_empty() {
                match self.next_arg() {
                    None      => {
                        // self.first was set to State::Start by next_arg.
                        Item::Error(ErrorKind::MissingParam(flag))
                    },
                    Some(arg) => {
                        self.first = InnerState::Start;
                        Item::Opt(flag, Some(arg))
                    },
                }
            } else {
                self.first = InnerState::Start;
                Item::Opt(flag, Some(rest))
            },
            Some(IfAttached) => {
                self.first = InnerState::Start;
                if rest.is_empty() {
                    Item::Opt(flag, None)
                } else {
                    Item::Opt(flag, Some(rest))
                }
            },
            Some(Never) => {
                self.first = InnerState::ShortOpts(rest);
                Item::Opt(flag, None)
            }
        }
    }

    fn next_arg(&mut self) -> Option<&'a str> {
        if let Some(arg) = self.rest.get(0) {
            self.rest = &self.rest[1 ..];
            Some(arg.borrow())
        } else {
            self.first = InnerState::Start;
            None
        }
    }
}

impl<'a, Cfg, Arg> Iterator for SliceIter<'a, Cfg, Arg>
    where Cfg: Config,
          Arg: Borrow<str> {

    type Item = Item<'a>;

    fn next(&mut self) -> Option<Item<'a>> {
        self.state.next_with_config(&self.config)
    }
}

impl<'a, Cfg, Arg> fmt::Debug for SliceIter<'a, Cfg, Arg>
    where Cfg: fmt::Debug,
          Arg: Borrow<str> {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SliceIter")
            .field("config", &self.config)
            .field("state", &self.state)
            .finish()
    }
}

impl<'a, Arg> fmt::Debug for State<'a, Arg>
    where Arg: Borrow<str> {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut list = f.debug_list();

        self.first.fmt_to_debug_list(&mut list);

        list.entries(self.rest.into_iter().map(Borrow::borrow));

        list.finish()
    }
}

