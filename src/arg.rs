use super::*;
use util::*;

use std::{fmt, io};

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
    descr:      String,
}

impl<'a, T> fmt::Debug for Arg<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Arg")
            .field("name",      &self.name)
            .field("action",    &"…")
            .field("short",     &self.short)
            .field("long",      &self.long)
            .field("descr",     &self.descr)
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
            descr:      String::new(),
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

    /// Sets the description of the option (for the help message).
    pub fn description<S: Into<String>>(mut self, s: S) -> Self {
        self.descr = s.into();
        self
    }

    pub (crate) fn new_error(&self, long: bool, msg: &str) -> Error {
        let opt_name = if long {
            format!("--{}", self.long)
        } else if let Some(c) = self.short {
            format!("-{}", c)
        } else {
            "-?".to_owned()
        };

        Error::from_string(msg).with_option(opt_name)
    }

    /// Writes the usage for this option to the writer.
    pub (crate) fn write_option_usage<W: io::Write>(&self, mut out: W) -> io::Result<()> {
        if self.is_positional() { return Ok(()); }

        if let Some(c) = self.short {
            if self.long.is_empty() {
                write!(out, "  -{}", c)?;
            } else {
                write!(out, "  -{}, --{}", c, self.long)?;
            }
        } else {
            write!(out, "  --{}", self.long)?;
        }

        if !self.name.is_empty() {
            write!(out, " <{}>", self.name)?;
        }

        if !self.descr.is_empty() {
            write!(out, "   {}", self.descr)?;
        }

        writeln!(out)
    }

    pub (crate) fn is_positional(&self) -> bool {
        self.short.is_none() && self.long.is_empty()
    }

    pub (crate) fn takes_parameter(&self) -> bool {
        !self.name.is_empty()
    }

    pub (crate) fn get_short(&self) -> Option<char> {
        self.short
    }

    pub (crate) fn get_long(&self) -> Option<&str> {
        non_empty_string(&self.long)
    }

    pub (crate) fn positional_name(&self) -> &str {
        static ARG: &'static str = "ARG";

        if self.name.is_empty() {
            ARG
        } else {
            &self.name
        }
    }

    /// Attempts to parse an argument.
    ///
    /// # Parameters
    ///
    /// `&self` – the formal `Arg` we are looking for
    ///
    /// `param` – the parameter supplied to the option, if any.
    pub (crate) fn parse_argument<'b>(&self, param: &'b str) -> Result<T> {
        (self.action)(param)
    }
}

