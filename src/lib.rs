use std::fmt;

#[derive(Debug)]
pub struct Builder<'a, T> {
    name:       String,
    version:    Option<String>,
    author:     Option<String>,
    about:      Option<String>,
    args:       Vec<Arg<'a, T>>,
}

/// A description of an argument, which may be a flag or have a parameter.
#[derive(Debug)]
pub struct Arg<'a, T> {
    parameter:  Option<Parameter<'a, T>>,
    short:      String,
    long:       String,
    equals:     String,
}

struct Parameter<'a, T> {
    name:       String,
    parser:     Box<Fn(&str) -> Result<T> + 'a>,
}

/// The result type for argument parsers.
pub type Result<T> = std::result::Result<T, Error>;

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
        if arg.short.is_empty() && arg.long.is_empty() && arg.parameter.is_none() {
            panic!("foropts::Builder::arg: nameless flag")
        }

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
    pub fn flag() -> Self {
        Arg {
            parameter: None,
            short:     String::new(),
            long:      String::new(),
            equals:    String::new(),
        }
    }

    pub fn param<S, F>(name: S, parser: F) -> Self
        where S: Into<String>,
              F: Fn(&str) -> Result<T> + 'a
    {
        let mut result = Self::flag();
        result.parameter = Some(Parameter {
            name: name.into(),
            parser: Box::new(parser),
        });
        result
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

impl<'a, T> fmt::Debug for Parameter<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Parameter")
            .field("name", &self.name)
            .field("parser", &"…")
            .finish()
    }
}

