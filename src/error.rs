use std::{fmt, result};

/// The result type for argument parsers.
pub type Result<T> = result::Result<T, Error>;

/// The error type for argument parser.
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Error {
    option:     String,
    message:    String,
}

impl Error {
    /// Creates an argument error from any type that can be stringified.
    pub fn from_string<S: ToString + ?Sized>(e: &S) -> Self {
        Error {
            option:    String::new(),
            message:   e.to_string(),
        }
    }

    /// Sets the particular option that triggered the error.
    pub fn with_option<S: Into<String>>(mut self, option: S) -> Self {
        self.option = option.into();
        self
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        "Argument parsing error"
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.option.is_empty() {
            write!(f, "option {}: ", self.option)?;
        }

        write!(f, "{}", self.message)
    }
}
