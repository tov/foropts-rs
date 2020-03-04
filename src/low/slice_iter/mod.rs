use super::super::util::split_first_str;

use std::borrow::Borrow;
use std::fmt;

mod errors;
mod item;

#[cfg(test)]
mod tests;

pub use self::errors::ErrorKind;
pub use self::item::Item;
pub use super::{Flag, Presence, Config, HashConfig, FnConfig};

use self::Presence::*;

#[derive(Clone)]
pub struct SliceIter<'a, Cfg, Arg: 'a> {
    config:     Cfg,
    state:      State<'a, Arg>,
}

#[derive(Clone)]
struct State<'a, Arg: 'a> {
    first:      InnerState<&'a str>,
    rest:       &'a [Arg],
}

#[derive(Clone)]
enum InnerState<S> {
    Start,
    ShortOpts(S),
    PositionalOnly,
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

    pub fn next_with_config<Cfg1: Config>(&mut self, config: Cfg1)
        -> Option<Item<'a, Cfg1::Token>> {

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
    pub fn next_with_config<Cfg: Config>(&mut self, config: Cfg) -> Option<Item<'a, Cfg::Token>> {
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

    fn parse_long<Cfg: Config>(&mut self, config: Cfg, after_hyphens: &'a str)
        -> Item<'a, Cfg::Token> {

        if let Some(index) = after_hyphens.find('=') {
            let long  = &after_hyphens[.. index];
            let param = &after_hyphens[index + 1 ..];
            let flag  = Flag::Long(long);
            match config.get_long_policy(long) {
                None         => Item::Error(ErrorKind::UnknownFlag(flag)),
                Some(policy) => match policy.presence {
                    Never      => Item::Error(ErrorKind::UnexpectedParam(flag, param)),
                    IfAttached => Item::Opt(flag, Some(param), policy.token),
                    Always     => Item::Opt(flag, Some(param), policy.token),
                },
            }
        } else {
            let long = after_hyphens;
            let flag = Flag::Long(long);
            match config.get_long_policy(long) {
                None             => Item::Error(ErrorKind::UnknownFlag(flag)),
                Some(policy)     => match policy.presence {
                    Never      => Item::Opt(flag, None, policy.token),
                    IfAttached => Item::Opt(flag, None, policy.token),
                    Always     => match self.next_arg() {
                        None           => Item::Error(ErrorKind::MissingParam(flag)),
                        Some(param)    => Item::Opt(flag, Some(param), policy.token),
                    },
                },
            }
        }
    }

    fn parse_short<Cfg: Config>(&mut self, config: Cfg, c: char, rest: &'a str)
        -> Item<'a, Cfg::Token> {

        let flag = Flag::Short(c);

        match config.get_short_policy(c) {
            None => {
                self.first = InnerState::ShortOpts(rest);
                Item::Error(ErrorKind::UnknownFlag(flag))
            },
            Some(policy) => match policy.presence {
                Always => if rest.is_empty() {
                    match self.next_arg() {
                        None      => {
                            // self.first was set to State::Start by next_arg.
                            Item::Error(ErrorKind::MissingParam(flag))
                        },
                        Some(arg) => {
                            self.first = InnerState::Start;
                            Item::Opt(flag, Some(arg), policy.token)
                        },
                    }
                } else {
                    self.first = InnerState::Start;
                    Item::Opt(flag, Some(rest), policy.token)
                },
                IfAttached => {
                    self.first = InnerState::Start;
                    if rest.is_empty() {
                        Item::Opt(flag, None, policy.token)
                    } else {
                        Item::Opt(flag, Some(rest), policy.token)
                    }
                },
                Never => {
                    self.first = InnerState::ShortOpts(rest);
                    Item::Opt(flag, None, policy.token)
                }
            },
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

    type Item = Item<'a, Cfg::Token>;

    fn next(&mut self) -> Option<Self::Item> {
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

impl<S: Borrow<str>> InnerState<S> {
    fn fmt_to_debug_list(&self, list: &mut fmt::DebugList) {
        match *self {
            InnerState::Start => (),
            InnerState::ShortOpts(ref shorts) => {
                list.entry(&format!("-{}", shorts.borrow()));
            }
            InnerState::PositionalOnly => {
                list.entry(&"--");
            }
        }
    }
}
