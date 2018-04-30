use super::*;

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

    fn add_arg(&mut self, arg: Arg<'a, T>) {
        for each in &self.args {
            if let (Some(c1), Some(c2)) = (each.get_short(), arg.get_short()) {
                assert_ne!( c1, c2, "foropts::Builder::arg: repeat of short option" );
            }

            if let (Some(s1), Some(s2)) = (each.get_long(), arg.get_long()) {
                assert_ne!( s1, s2, "foropts::Builder::arg: repeat of long option" );
            }
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
        Iter::new(self, args)
    }

    pub (crate) fn parse_positional(&self, arg: &str) -> Option<Result<T>> {
        for each in &self.args {
            if let Some(result) = each.parse_positional(arg) {
                return Some(result)
            }
        }

        None
    }

    pub (crate) fn get_args(&self) -> &[Arg<'a, T>] {
        &self.args
    }
}


