//! A low-level, borrowing argument parser.

mod common;
mod config;
mod errors;
mod flag;
mod item;
mod iter_iter;
mod policy;
mod slice_iter;

#[cfg(test)]
mod tests;

pub use self::config::{Config, HashConfig, TokenHashConfig, FnConfig, TokenFnConfig};
pub use self::errors::ErrorKind;
pub use self::flag::Flag;
pub use self::item::Item;
pub use self::iter_iter::ArgItem;
pub use self::policy::{Presence, Policy};
pub use self::slice_iter::SliceIter;
