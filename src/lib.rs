use std::fmt;
use std::mem;
use std::result;
use std::str::FromStr;

#[derive(Debug)]
pub struct Builder<'a, T> {
    name:       String,
    version:    Option<String>,
    author:     Option<String>,
    about:      Option<String>,
    args:       Vec<Arg<'a, T>>,
}

/// A description of an argument, which may be a flag or have a parameter.
pub struct Arg<'a, T> {
    name:       String,
    action:     Box<Fn(&str) -> Result<T> + 'a>,
    short:      Option<char>,
    long:       String,
}

/// The result type for argument parsers.
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub struct Error {
    message: String,
}

#[derive(Debug)]
pub struct Iter<'a, 'b: 'a, I, T: 'a>
    where I: IntoIterator<Item=String>
{
    config:     &'a Builder<'b, T>,
    args:       I::IntoIter,
    push_back:  Option<String>,
    positional: bool,
}

impl<'a, 'b, I, T> Iterator for Iter<'a, 'b, I, T>
    where I: IntoIterator<Item=String>
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Result<T>> {
        self.push_back.take().or_else(|| self.args.next()).and_then(|item| {
            let mut arg = item.as_str();

            if let Some(c) = arg.chars().next() {
                if c == '-' {
                    arg = &arg[1..];

                    for each in &self.config.args {
                        if let Some(result) = each.parse_optional(&mut arg, &mut self.args) {
                            if !arg.is_empty() {
                                self.push_back = Some(format!("-{}", arg));
                            }

                            return Some(result);
                        }
                    }

                    let msg = format!("Unknown option: -{}", arg);
                    Some(Err(Error::from_string(msg)))
                } else {
                    self.config.parse_positional(arg)
                }
            } else {
                self.config.parse_positional("")
            }
        })
    }
}

impl<'a, T> Builder<'a, T> {
    /// Creates a new `foropts::Builder` given the name of the program.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Builder {
            name:    name.into(),
            version: None,
            author:  None,
            about:   None,
            args:    Vec::new(),
        }
    }

    /// Sets the program name.
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the version string.
    pub fn version<S: Into<String>>(mut self, version: S) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Sets the author string.
    pub fn author<S: Into<String>>(mut self, author: S) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Sets the description string.
    pub fn about<S: Into<String>>(mut self, about: S) -> Self {
        self.about = Some(about.into());
        self
    }

    fn add_arg(&mut self, arg: Arg<'a, T>) {
        for each in &self.args {
            match (each.short, arg.short) {
                (Some(c1), Some(c2)) =>
                    assert_ne!( c1, c2, "foropts::Builder::arg: repeat of short option" ),
                _ => (),
            }
            assert_ne!( each.long, arg.long,
                        "foropts::Builder::arg: repeat of long option" );
        }

        self.args.push(arg);
    }

    /// Adds an argument to the list of arguments.
    pub fn arg(mut self, arg: Arg<'a, T>) -> Self {
        self.add_arg(arg);
        self
    }

    /// Adds a arguments to the list of arguments.
    pub fn args<I: IntoIterator<Item=Arg<'a, T>>>(mut self, args: I) -> Self {
        for arg in args {
            self.add_arg(arg);
        }
        self
    }

    /// Given an iterator over the arguments, returns an iterator that parses the results.
    pub fn iter<'b, I: IntoIterator<Item=String>>(&'b self, args: I) -> Iter<'b, 'a, I, T> {
        Iter {
            config:     self,
            args:       args.into_iter(),
            push_back:  None,
            positional: false,
        }
    }

    fn parse_positional(&self, arg: &str) -> Option<Result<T>> {
        for each in &self.args {
            if let Some(result) = each.parse_positional(arg) {
                return Some(result)
            }
        }

        None
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

    fn parse_positional(&self, arg: &str) -> Option<Result<T>> {
        if self.short.is_none() && self.long.is_empty() {
            Some((self.action)(arg))
        } else {None}
    }

    fn parse_optional<I>(&self, arg: &mut &str, rest: &mut I) -> Option<Result<T>>
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
        } else {
            if self.name.is_empty() {
                *arg = "";
                Some((self.action)("-"))
            } else {
                None
            }
        }
    }
}

impl Error {
    pub fn from_string<S: ToString>(e: S) -> Self {
        Error {
            message: e.to_string(),
        }
    }
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

pub fn parse_map<'a, A, B, F>(slice: &'a str, success: F) -> Result<B>
    where A: FromStr,
          A::Err: ToString,
          F: FnOnce(A) -> B
{
    slice.parse().map(success).map_err(Error::from_string)
}

#[cfg(test)]
mod tests {
    use super::{Builder, Arg, parse_map, Result};

    #[derive(Clone, PartialEq, Debug)]
    enum Opt {
        Louder,
        Softer,
        Freq(f32),
    }

    #[test]
    fn flag_s() {
        assert_parse(&["-s"], &[Opt::Softer]);
    }

    #[test]
    fn flag_s_s() {
        assert_parse(&["-ss"], &[Opt::Softer, Opt::Softer]);
    }

    #[test]
    fn flag_softer() {
        assert_parse(&["--softer"], &[Opt::Softer]);
    }

    #[test]
    fn flag_s_l_s() {
        let expected = &[Opt::Softer, Opt::Louder, Opt::Softer];
        assert_parse(&["-sls"], expected);
        assert_parse(&["-s", "-ls"], expected);
        assert_parse(&["-sl", "-s"], expected);
        assert_parse(&["-s", "-l", "-s"], expected);
    }

    #[test]
    fn flag_f_needs_param() {
        assert_parse_error(&["-f"]);
    }

    #[test]
    fn flag_freq_needs_param() {
        assert_parse_error(&["--freq"]);
    }

    #[test]
    fn flag_freq_needs_float_param() {
        assert_parse_error(&["-fhello"]);
        assert_parse_error(&["-f", "hello"]);
        assert_parse_error(&["--freq=hello"]);
        assert_parse_error(&["--freq", "hello"]);

        assert_parse(&["-f5.5"], &[Opt::Freq(5.5)]);
        assert_parse(&["-f", "5.5"], &[Opt::Freq(5.5)]);
        assert_parse(&["--freq=5.5"], &[Opt::Freq(5.5)]);
        assert_parse(&["--freq", "5.5"], &[Opt::Freq(5.5)]);
    }

    fn assert_parse_error(args: &[&str]) {
        assert!( parse(args).is_err() );
    }

    fn assert_parse(args: &[&str], expected: &[Opt]) {
        assert_eq!( parse(args), Ok(expected.to_owned()) );
    }

    fn parse(args: &[&str]) -> Result<Vec<Opt>> {
        let builder = get_builder();
        let args = args.into_iter().map(ToString::to_string);
        builder.iter(args).collect()
    }

    fn get_builder() -> Builder<'static, Opt> {
        Builder::new("moo")
            .arg(Arg::flag(|| Opt::Louder).short('l').long("louder"))
            .arg(Arg::flag(|| Opt::Softer).short('s').long("softer"))
            .arg(Arg::param("FREQ", |s| parse_map(s, Opt::Freq)).short('f').long("freq"))
    }
}
