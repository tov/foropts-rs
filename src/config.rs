use super::*;

use std::collections::hash_map::{self, HashMap};
use std::io;
use std::process::exit;

/// The configuration for the argument parser.
///
/// # Parameters
///
/// `<'a>` – The lifetime of the arguments’ actions
///
/// `<T>`  – The result type that each argument will be parsed into
#[derive(Debug)]
pub struct Config<'a, T> {
    name:       String,
    version:    Option<String>,
    author:     Option<String>,
    about:      Option<String>,
    args:       Vec<Arg<'a, T>>,
    short_map:  HashMap<char, usize>,
    long_map:   HashMap<String, usize>,
    positional: Option<Arg<'a, T>>,
}

impl<'a, T> Config<'a, T> {
    /// Creates a new `foropts::Builder` given the name of the program.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Config {
            name:       name.into(),
            version:    None,
            author:     None,
            about:      None,
            args:       Vec::new(),
            short_map:  HashMap::new(),
            long_map:   HashMap::new(),
            positional: None,
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
        use self::hash_map::Entry::*;

        if arg.is_positional() {
            if self.positional.is_none () {
                self.positional = Some(arg);
                return Ok(());
            } else {
                return Err(Error::from_string("multiple positional arguments"))
            }
        }

        let index = self.args.len();

        if let Some(c) = arg.get_short() {
            match self.short_map.entry(c) {
                Vacant(entry) => {
                    entry.insert(index);
                }
                Occupied(_)   => {
                    return Err(Error::from_string("repeated in config")
                        .with_option(format!("-{}", c)));
                }
            }
        }

        if let Some(s) = arg.get_long() {
            match self.long_map.entry(s.to_owned()) {
                Vacant(entry) => {
                    entry.insert(index);
                }
                Occupied(_)   => {
                    return Err(Error::from_string("repeated in config")
                        .with_option(format!("--{}", s)));
                }
            }
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

    /// Given an iterator over the unparsed arguments, returns an iterator over the
    /// parsed arguments.
    pub fn iter<'b, I: IntoIterator<Item=String>>(&'b self, args: I) -> Iter<'b, 'a, I, T> {
        Iter::new(self, args)
    }

    /// Exits with an error message and usage information printed on stderr,
    /// with exit code 1.
    pub fn exit_error(&self, error: &Error) -> ! {
        eprintln!("Syntax error: {}", error);
        self.write_usage(io::stderr()).unwrap();
        exit(1);
    }

    /// Prints usage information to stdout and exits with code 0.
    pub fn exit_usage(&self) -> ! {
        let stdout = io::stdout();
        self.write_usage(stdout.lock()).unwrap();
        exit(0);
    }

    /// Prints version information to stdout and exits with code 0.
    pub fn exit_version(&self) -> ! {
        let stdout = io::stdout();
        self.write_version(stdout.lock()).unwrap();
        exit(0);
    }

    /// Writes version information to the given `Write`.
    pub fn write_version<W: io::Write>(&self, mut out: W) -> io::Result<()> {
        write!(out, "{}", self.name)?;
        if let Some(ref version) = self.version {
            write!(out, " {}", *version)?;
        }
        writeln!(out)
    }

    fn write_usage_line<W: io::Write>(&self, mut out: W) -> io::Result<()> {
        write!(out, "Usage: {} OPTION...", self.name)?;

        if let Some(ref arg) = self.positional {
            writeln!(out, " [--] {}...", arg.positional_name())
        } else {
            writeln!(out)
        }
    }

    /// Writes usage information to the given `Write`.
    pub fn write_usage<W: io::Write>(&self, mut out: W) -> io::Result<()> {
        self.write_version(&mut out)?;
        if let Some(ref author) = self.author {
            writeln!(out, "{}", *author)?;
        }
        if let Some(ref about) = self.about {
            writeln!(out, "{}", *about)?;
        }
        writeln!(out)?;

        self.write_usage_line(&mut out)?;

        writeln!(out, "\nOPTIONS:")?;
        for arg in &self.args {
            arg.write_option_usage(&mut out)?;
        }
        Ok(())
    }

    pub (crate) fn get_positional(&self) -> Option<&Arg<'a, T>> {
        self.positional.as_ref()
    }

    pub (crate) fn get_short(&self, c: char) -> Option<&Arg<'a, T>> {
        self.short_map.get(&c).map(|i| &self.args[*i])
    }

    pub (crate) fn get_long(&self, s: &str) -> Option<&Arg<'a, T>> {
        self.long_map.get(s).map(|i| &self.args[*i])
    }
}

