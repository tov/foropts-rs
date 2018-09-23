//! A low-level, borrowing argument parser.

mod common;
mod config;
mod errors;
mod flag;
mod item;
mod iter_iter;
mod owned_item;
mod policy;
mod slice_iter;

#[cfg(test)]
mod tests;

pub use self::config::{Config, HashConfig0, HashConfig, FnConfig0, FnConfig};
pub use self::errors::ErrorKind;
pub use self::flag::Flag;
pub use self::item::Item;
pub use self::policy::{Presence, Policy};
pub use self::owned_item::{OwnedItem, FlagOpt, ParamOpt, Positional};
pub use self::slice_iter::SliceIter;
