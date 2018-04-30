use std::{fmt, result};

/// The result type for argument parsers.
pub type Result<T> = result::Result<T, Error>;

/// The error type for argument parser.
#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    message: String,
}

impl Error {
    /// Creates an argument error from any type that can be stringified.
    pub fn from_string<S: ToString>(e: &S) -> Self {
        Error {
            message: e.to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.message)
    }
}