//! A low-level, borrowing argument parser.

mod common;
mod config;
mod errors;
mod flag;
mod item;
mod slice_iter;

#[cfg(test)]
mod tests;

pub use self::config::{Presence, Config, HashConfig, FnConfig};
pub use self::errors::ErrorKind;
pub use self::flag::Flag;
pub use self::item::Item;
pub use self::slice_iter::SliceIter;
