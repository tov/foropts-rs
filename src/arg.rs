use util::*;
use super::*;

use std::fmt;

/// A description of an argument, which may be a Boolean flag or carry a parameter.
///
/// # Parameters
///
/// `<'a>` – The lifetime of the argument’s action
///
/// `<T>`  – The result type of the argument
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
    /// Creates a new Boolean flag whose action is the given thunk.
    pub fn flag<F>(thunk: F) -> Self
        where F: Fn() -> T + 'a
    {
        Self::str_param("", move |_| Ok(thunk()))
    }

    /// Creates a new argument with raw string parameter.
    ///
    /// # Parameters
    ///
    /// `<S>` – type converted to `String` to name the parameter
    ///
    /// `<F>` – type of parsing function
    ///
    /// `name` – the name of the parameter
    ///
    /// `parser` – the parsing function, which must convert the raw
    /// `&str` to a `Result<T>`
    pub fn str_param<S, F>(name: S, parser: F) -> Self
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

    /// Creates a new argument with a parameter parsed by `str::parse`.
    ///
    /// # Parameters
    ///
    /// `<A>` – type into which the raw `&str` will be automatically parsed
    ///
    /// `<S>` – type converted to `String` to name the parameter
    ///
    /// `<F>` – type of the wrapping function
    ///
    /// `name` – the name of the parameter
    ///
    /// `wrapper` – function applied to successful parsing result to transform `A` to `T`
    pub fn parsed_param<A, S, F>(name: S, wrapper: F) -> Self
        where S: Into<String>,
              F: Fn(A) -> T + 'a,
              A: FromStr,
              A::Err: ToString
    {
        Arg::str_param(name, move |slice|
            slice.parse()
                .map(&wrapper)
                .map_err(|s| Error::from_string(&s)))
    }

    /// Sets the short name of the option.
    pub fn short(mut self, c: char) -> Self {
        assert_ne!( c, '-' , "Arg::short: c cannot be '-'" );
        self.short = Some(c);
        self
    }

    /// Sets the long name of the option.
    pub fn long<S: Into<String>>(mut self, s: S) -> Self {
        self.long = s.into();
        self
    }

    /// Checks that this `Arg` and another `Arg` don't claim the same option names.
    pub (crate) fn check_interference<'b, U>(&self, other: &Arg<'b, U>) -> Result<()> {
        if let (Some(c1), Some(c2)) = (self.short, other.short) {
            if c1 == c2 {
                return Err(Error::from_string("repeated in config")
                               .with_option(format!("-{}", c1)))
            }
        }

        let longs = (non_empty_string(&self.long), non_empty_string(&other.long));
        if let (Some(s1), Some(s2)) = longs {
            if s1 == s2 {
                return Err(Error::from_string("repeated in config")
                    .with_option(format!("--{}", s1)))
            }
        }

        Ok(())
    }

    /// Parses a position parameter, provided the given `Arg` accepts one.
    ///
    /// An `Arg` accepts a positional parameter when it has no short nor
    /// long option names.
    pub (crate) fn parse_positional(&self, arg: &str) -> Option<Result<T>> {
        if self.short.is_none() && self.long.is_empty() {
            Some((self.action)(arg))
        } else {None}
    }

    /// Attempts to parse an optional (non-positional) parameter.
    ///
    /// # Parameters
    ///
    /// `<I>` – the underlying iterator type
    ///
    /// `&self` – the formal `Arg` we are looking for
    ///
    /// `arg` – the actual argument we are are attempting to parse
    ///
    /// `rest` – the iterator from which to extract additional raw arguments
    ///
    /// Note that `arg` should not include the leading hyphen (`'-'`), but should include the
    /// second hyphen if it’s a long argument. The pointer `arg` will be updated to contain a slice
    /// with any remaining, unprocessed portion of the argument.
    pub (crate) fn parse_optional<'b, I>(&self, arg: &'b str, args: &mut I)
        -> Option<(Result<T>, &'b str)>
        where I: Iterator<Item=String>
    {
        if let Some((c, rest)) = split_first_str(arg) {
            if c == '-' {
                if rest == self.long {
                    let result = if self.name.is_empty() {
                        (self.action)("")
                    } else if let Some(next) = args.next() {
                        (self.action)(&next)
                    } else {
                        Err(Error::from_string("requires parameter"))
                    };
                    Some((result, ""))
                } else if let Some(index) = rest.find('=') {
                    if rest[0..index] == self.long {
                        let param = &rest[index + 1..];
                        Some(((self.action)(param), ""))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else if Some(c) == self.short {
                let pair = if self.name.is_empty() {
                    ((self.action)(""), rest)
                } else if rest.is_empty() {
                    if let Some(next) = args.next() {
                        ((self.action)(&next), "")
                    } else {
                        (Err(Error::from_string("requires parameter")), "")
                    }
                } else {
                    ((self.action)(rest), "")
                };
                Some(pair)
            } else {
                None
            }
        } else if self.name.is_empty() {
            Some(((self.action)("-"), ""))
        } else {
            None
        }.map(|(result, rest)| (result.map_err(|e| e.with_option(arg)), rest))
    }
}

