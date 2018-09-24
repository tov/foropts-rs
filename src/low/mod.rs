//! A low-level, borrowing argument parser.

    mod config;
    mod flag;
pub mod iter_iter;
    mod policy;
pub mod slice_iter;

pub use self::config::{Config, HashConfig, FnConfig};
pub use self::flag::Flag;
pub use self::policy::{Presence, Policy};
