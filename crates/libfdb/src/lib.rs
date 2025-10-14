//! Core library facade for the fdb debugger.
//! Provides modules reused by the CLI front end.

pub mod errors;
pub mod process;

pub use errors::{FdbError, FdbResult};

/// Exposes the crate version for CLI reporting.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
