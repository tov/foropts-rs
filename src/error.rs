use std::result;

/// The result type for argument parsers.
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn from_string<S: ToString>(e: &S) -> Self {
        Error {
            message: e.to_string(),
        }
    }
}

