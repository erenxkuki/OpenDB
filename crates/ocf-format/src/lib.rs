//! ocf-format — ODCF container format (Phase 2 prototype).

#![deny(unsafe_code)]

pub mod container;
pub mod error;

pub use container::*;
pub use error::*;
