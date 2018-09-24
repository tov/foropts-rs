mod item;
mod errors;

pub use self::item::{Item, Opt};
pub use self::errors::{Error, ErrorKind};
pub use super::{Flag, Presence, Config, HashConfig, FnConfig};

use std::borrow::Borrow;
use std::mem::replace;
use std::ops::{Deref, Range};

use self::Presence::*;

pub struct IterIter<Cfg, Src> where Src: Iterator {
    config: Cfg,
    state:  State<Src>,
}

impl<Cfg, Src> IterIter<Cfg, Src>
    where Cfg: Config,
          Src: Iterator,
          Src::Item: Borrow<str> {

    pub fn new(config: Cfg, source: Src) -> Self {
        IterIter {
            config,
            state: State {
                inner: InnerState::Start,
                iter:  source,
            },
        }
    }

    pub fn config_mut(&mut self) -> &mut Cfg {
        &mut self.config
    }


    pub fn next_with_config<Cfg1: Config>(&mut self, config: Cfg1)
        -> Option<Item<Src::Item, Cfg1::Token>> {

        self.state.next_with_config(config)
    }

    pub fn with_config<Cfg1>(self, config: Cfg1) -> IterIter<Cfg1, Src> {
        IterIter {
            config,
            state: self.state,
        }
    }
}

impl<Cfg, Src> Iterator for IterIter<Cfg, Src>
    where Cfg: Config,
          Src: Iterator,
          Src::Item: Borrow<str> {

    type Item = Item<Src::Item, Cfg::Token>;

    fn next(&mut self) -> Option<Item<Src::Item, Cfg::Token>> {
        self.state.next_with_config(&self.config)
    }
}

impl<Cfg, Src> Clone for IterIter<Cfg, Src>
    where Cfg: Clone,
          Src: Clone + Iterator,
          Src::Item: Clone {

    fn clone(&self) -> Self {
        IterIter {
            config: self.config.clone(),
            state:  self.state.clone(),
        }
    }
}

struct State<Src> where Src: Iterator {
    inner:  InnerState<Src::Item>,
    iter:   Src,
}

impl<Src> Clone for State<Src>
    where Src: Clone + Iterator,
          Src::Item: Clone {

    fn clone(&self) -> Self {
        State {
            inner: self.inner.clone(),
            iter:  self.iter.clone(),
        }
    }
}

#[derive(Clone)]
enum InnerState<S> {
    Start,
    ShortOpts(OwnedSlice<S>),
    PositionalOnly,
}

impl<Src> State<Src>
    where Src: Iterator,
          Src::Item: Borrow<str> {

    pub fn next_with_config<Cfg: Config>(&mut self, config: Cfg)
        -> Option<Item<Src::Item, Cfg::Token>> {

        loop {
            match replace(&mut self.inner, InnerState::Start) {
                InnerState::Start => {
                    match self.next_arg() {
                        None => return None,

                        Some(mut arg) => match arg.next() {
                            Some('-') => match arg.next() {
                                None => return Some(Item::Positional(arg.owned)),
                                Some('-') => if arg.is_empty() {
                                    self.inner = InnerState::PositionalOnly;
                                } else {
                                    return Some(self.parse_long(config, arg));
                                },
                                _ => self.inner = InnerState::ShortOpts(arg),
                            },
                            _ => return Some(Item::Positional(arg.owned)),
                        },
                    }
                }

                InnerState::ShortOpts(mut short) => {
                    if let Some(c) = short.next() {
                        return Some(self.parse_short(config, c, short));
                    }
                }

                InnerState::PositionalOnly => {
                    self.inner = InnerState::PositionalOnly;
                    return self.next_arg().map(|os| Item::Positional(os.owned));
                }
            }
        }
    }

    fn parse_long<Cfg: Config>(&mut self, config: Cfg, after_hyphens: OwnedSlice<Src::Item>)
        -> Item<Src::Item, Cfg::Token> {

        if let Some(index) = after_hyphens.find('=') {
            let policy_opt  = config.get_long_policy(&after_hyphens[.. index]);
            let range       = after_hyphens.range;
            let flag_range  = range.start .. range.start + index;
            let param_range = range.start + index + 1 .. range.end;
            let opt         = Opt::new_long_param(None, flag_range,
                                                  after_hyphens.owned, param_range);
            match policy_opt {
                None         => Item::error(ErrorKind::UnknownFlag, opt),
                Some(policy) => match policy.presence {
                    Never      => Item::error(ErrorKind::UnexpectedParam, opt),
                    IfAttached => Item::Opt(opt, policy.token),
                    Always     => Item::Opt(opt, policy.token),
                },
            }
        } else {
            let policy_opt  = config.get_long_policy(&after_hyphens[..]);
            let flag        = after_hyphens;
            match policy_opt {
                None         => Item::error(ErrorKind::UnknownFlag,
                                            Opt::new_long_flag(flag.owned, flag.range)),
                Some(policy) => match policy.presence {
                    Never      => Item::Opt(Opt::new_long_flag(flag.owned, flag.range),
                                            policy.token),
                    IfAttached => Item::Opt(Opt::new_long_flag(flag.owned, flag.range),
                                            policy.token),
                    Always     => match self.next_arg() {
                        None        => Item::error(ErrorKind::MissingParam,
                                                   Opt::new_long_flag(flag.owned, flag.range)),
                        Some(param) =>
                            Item::Opt(Opt::new_long_param(Some(flag.owned), flag.range,
                                                          param.owned, param.range),
                                      policy.token),
                    },
                }
            }
        }
    }

    fn parse_short<Cfg: Config>(&mut self, config: Cfg, c: char, rest: OwnedSlice<Src::Item>)
        -> Item<Src::Item, Cfg::Token> {

        let flag = Opt::new_short_flag(c);

        match config.get_short_policy(c) {
            None         => {
                self.inner = InnerState::ShortOpts(rest);
                Item::error(ErrorKind::UnknownFlag, flag)
            },

            Some(policy) => match policy.presence {
                Always     => if rest.is_empty() {
                    match self.next_arg() {
                        None        => Item::error(ErrorKind::MissingParam, flag),
                        Some(param) => Item::Opt(Opt::new_short_param(c, param.owned, param.range),
                                                 policy.token),
                    }
                } else {
                    Item::Opt(Opt::new_short_param(c, rest.owned, rest.range), policy.token)
                },

                IfAttached => if rest.is_empty() {
                    Item::Opt(flag, policy.token)
                } else {
                    Item::Opt(Opt::new_short_param(c, rest.owned, rest.range), policy.token)
                },

                Never      => {
                    self.inner = InnerState::ShortOpts(rest);
                    Item::Opt(flag, policy.token)
                }
            },
        }
    }

    fn next_arg(&mut self) -> Option<OwnedSlice<Src::Item>> {
        self.iter.next().map(OwnedSlice::new)
    }
}

#[derive(Clone)]
struct OwnedSlice<S> {
    owned:  S,
    range:  Range<usize>,
}

impl<S> OwnedSlice<S> where S: Borrow<str> {
    fn new(owned: S) -> Self {
        let range = 0 .. owned.borrow().len();
        OwnedSlice { owned, range, }
    }

    fn next(&mut self) -> Option<char> {
        if self.range.start < self.range.end {
            let remaining = &self.owned.borrow()[self.range.clone()];
            let mut chars = remaining.chars();
            let result = chars.next();
            self.range.start = self.range.end - chars.as_str().len();
            result
        } else {
            None
        }
    }
}

impl<S> Deref for OwnedSlice<S> where S: Borrow<str> {
    type Target = str;

    fn deref(&self) -> &str {
        &self.owned.borrow()[self.range.clone()]
    }
}

