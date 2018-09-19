//! A low-level, borrowing argument parser.

mod config;
mod flag;
mod iter;

#[cfg(test)]
mod tests;

pub use self::config::{Config, HashConfig, Presence};
pub use self::flag::Flag;
pub use self::iter::{Iter, Item, ErrorKind};
