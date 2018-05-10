use std::{fmt, result};

/// The result type for argument parsers.
pub type Result<T> = result::Result<T, Error>;

/// The error type for argument parser.
#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    original:   String,
    message:    String,
}

impl Error {
    /// Creates an argument error from any type that can be stringified.
    pub fn from_string<S: ToString + ?Sized>(e: &S) -> Self {
        Error {
            original:   String::new(),
            message:    e.to_string(),
        }
    }

    /// Sets the particular option that triggered the error.
    pub fn with_option<S: Into<String>>(mut self, option: S) -> Self {
        self.original = option.into();
        self
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.original.is_empty() {
            write!(f, "option {}: ", self.original)?;
        }
        f.write_str(&self.message)
    }
}
