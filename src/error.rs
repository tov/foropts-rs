use std::{fmt, result};

/// The result type for argument parsers.
pub type Result<T> = result::Result<T, Error>;

/// The error type for argument parser.
#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    message:    String,
}

impl Error {
    /// Creates an argument error from any type that can be stringified.
    pub fn from_string<S: ToString + ?Sized>(e: &S) -> Self {
        Error {
            message:   e.to_string(),
        }
    }

    /// Sets the particular option that triggered the error.
    pub fn with_option<S: Into<String>>(mut self, option: S) -> Self {
        if !self.message.starts_with("option -") {
            self.message = format!("option {}: {}", option.into(), self.message);
        }

        self
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
