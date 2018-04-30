use util::*;
use super::*;

use std::{fmt, mem};

/// A description of an argument, which may be a flag or have a parameter.
pub struct Arg<'a, T> {
    name:       String,
    action:     Box<Fn(&str) -> Result<T> + 'a>,
    short:      Option<char>,
    long:       String,
}

impl<'a, T> fmt::Debug for Arg<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Arg")
            .field("name",      &self.name)
            .field("action",    &"…")
            .field("short",     &self.short)
            .field("long",      &self.long)
            .finish()
    }
}

impl<'a, T> Arg<'a, T> {
    pub fn flag<F>(thunk: F) -> Self
        where F: Fn() -> T + 'a
    {
        Self::param("", move |_| Ok(thunk()))
    }

    pub fn param<S, F>(name: S, parser: F) -> Self
        where S: Into<String>,
              F: Fn(&str) -> Result<T> + 'a
    {
        Arg {
            name:       name.into(),
            action:     Box::new(parser),
            short:      None,
            long:       String::new(),
        }
    }

    /// Sets the short name of the option.
    pub fn short(mut self, c: char) -> Self {
        assert_ne!( c, '-' , "Arg::short: c cannot be '-'" );
        self.short = Some(c);
        self
    }

    /// Sets the long name of the option.
    pub fn long<'b, S: Into<String>>(mut self, s: S) -> Self {
        self.long = s.into();
        self
    }

    pub (crate) fn check_interference<'b, U>(&self, other: &Arg<'b, U>) -> Result<()> {
        if let (Some(c1), Some(c2)) = (self.short, other.short) {
            if c1 == c2 {
                let msg = format!("repeat of short option: -{}", c1);
                return Err(Error::from_string(&msg));
            }
        }

        let longs = (non_empty_string(&self.long), non_empty_string(&other.long));
        if let (Some(s1), Some(s2)) = longs {
            if s1 == s2 {
                let msg = format!("repeat of long option: --{}", s1);
                return Err(Error::from_string(&msg));
            }
        }

        Ok(())
    }

    pub (crate) fn parse_positional(&self, arg: &str) -> Option<Result<T>> {
        if self.short.is_none() && self.long.is_empty() {
            Some((self.action)(arg))
        } else {None}
    }

    pub (crate) fn parse_optional<I>(&self, arg: &mut &str, rest: &mut I) -> Option<Result<T>>
        where I: Iterator<Item=String>
    {
        if let Some(c) = arg.chars().next() {
            if c == '-' {
                if &arg[1..] == self.long {
                    *arg = "";
                    if self.name.is_empty() {
                        Some((self.action)(""))
                    } else if let Some(next) = rest.next() {
                        Some((self.action)(&next))
                    } else {
                        let msg = format!("Option --{} requires parameter", self.long);
                        Some(Err(Error::from_string(msg)))
                    }
                } else if let Some(index) = arg.find('=') {
                    if &arg[1..index] == self.long {
                        let param = &arg[index + 1..];
                        *arg = "";
                        Some((self.action)(param))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else if Some(c) == self.short {
                *arg = &arg[1..];
                if self.name.is_empty() {
                    Some((self.action)(""))
                } else if arg.is_empty() {
                    if let Some(next) = rest.next() {
                        Some((self.action)(&next))
                    } else {
                        let msg = format!("Option -{} requires parameter", c);
                        Some(Err(Error::from_string(msg)))
                    }
                } else {
                    let param = mem::replace(arg, "");
                    Some((self.action)(param))
                }
            } else {
                None
            }
        } else if self.name.is_empty() {
            *arg = "";
            Some((self.action)("-"))
        } else {
            None
        }
    }
}

