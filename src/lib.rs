use std::fmt;
use std::result;

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
    short:      String,
    long:       String,
    equals:     String,
}

/// The result type for argument parsers.
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    message: String,
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
//        if arg.short.is_empty() && arg.long.is_empty() {
//            panic!("foropts::Builder::arg: nameless flag")
//        }

        for each in &self.args {
            assert_ne!( each.short, arg.short,
                        "foropts::Builder::arg: repeat of short option" );
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
            short:      String::new(),
            long:       String::new(),
            equals:     String::new(),
        }
    }

    /// Sets the short name of the option.
    pub fn short(mut self, c: char) -> Self {
        assert_ne!( c, '-' , "Arg::short: c cannot be '-'" );
        self.short = format!("-{}", c);
        self
    }

    /// Sets the long name of the option.
    pub fn long<'b, S: Into<&'b str>>(mut self, s: S) -> Self {
        let s = s.into();
        self.long = format!("--{}", s);
        self.equals = format!("--{}=", s);
        self
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
            .field("equals",    &self.equals)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{Builder, Arg, Error};

    enum MooOpt {
        Louder,
        Softer,
        Freq(f32),
    }

    #[test]
    fn moo() {
        let _builder = Builder::new("moo")
            .arg(Arg::flag(|| MooOpt::Louder).short('l').long("louder"))
            .arg(Arg::flag(|| MooOpt::Softer).short('s').long("softer"))
            .arg(Arg::param("FREQ", |s|
                s.parse()
                    .map(MooOpt::Freq)
                    .map_err(Error::from_string))
                .short('f').long("freq"));
    }
}
