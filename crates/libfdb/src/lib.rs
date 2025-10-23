//! Core library facade for the fdb debugger.
//! Provides modules reused by the CLI front end.

pub mod errors;
pub mod process;

pub use errors::{FdbError, FdbResult};

/// Exposes the crate version for CLI reporting.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(Debug, Clone, Copy)]
pub enum ProcessState {
    /// Created/attached but not yet continued by us
    Initialized,
    /// Currently running after a continue
    Running,
    /// The process stopped and why
    Stopped(StopReason),
    /// The process exited with code
    Exited(i32),
    /// The process was terminated by a signal
    Terminated(nix::sys::signal::Signal),
}

#[derive(Debug, Clone, Copy)]
pub struct StopReason {
    /// Which signal caused the stop (SIGTRAP, SIGINT, etc.)
    pub signal: nix::sys::signal::Signal,
}
