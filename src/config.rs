use super::*;

/// The configuration for the argument parser.
#[derive(Debug)]
pub struct Config<'a, T> {
    name:       String,
    version:    Option<String>,
    author:     Option<String>,
    about:      Option<String>,
    args:       Vec<Arg<'a, T>>,
}

impl<'a, T> Config<'a, T> {
    /// Creates a new `foropts::Builder` given the name of the program.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Config {
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

    /// Adds an argument to the list of arguments, returning `Result::Err` if the
    /// argument cannot be added.
    pub fn arg_safe(&mut self, arg: Arg<'a, T>) -> Result<()> {
        for each in &self.args {
            each.check_interference(&arg)?;
        }

        self.args.push(arg);

        Ok(())
    }

    /// Adds an argument to the list of arguments.
    ///
    /// # Panics
    ///
    /// Panics if the argument cannot be added.
    pub fn arg(mut self, arg: Arg<'a, T>) -> Self {
        self.arg_safe(arg).expect("foropts::Arg::arg: repeated arg");
        self
    }

    /// Adds arguments to the list of arguments.
    ///
    /// # Panics
    ///
    /// Panics if an argument cannot be added.
    pub fn args<I: IntoIterator<Item=Arg<'a, T>>>(mut self, args: I) -> Self {
        for arg in args {
            self.arg_safe(arg).expect("foropts::Arg::args: repeated arg");
        }
        self
    }

    /// Given an iterator over the arguments, returns an iterator that parses the results.
    pub fn iter<'b, I: IntoIterator<Item=String>>(&'b self, args: I) -> Iter<'b, 'a, I, T> {
        Iter::new(self, args)
    }

    pub (crate) fn parse_positional(&self, arg: &str) -> Option<Result<T>> {
        self.args.iter()
            .flat_map(|each| each.parse_positional(arg))
            .next()
    }

    pub (crate) fn get_args(&self) -> &[Arg<'a, T>] {
        &self.args
    }
}

