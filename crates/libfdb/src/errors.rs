//! Error taxonomy for the debugger core.

use nix::errno;
use std::ffi::NulError;
use thiserror::Error;

/// Unified result type across the debugger core.
pub type FdbResult<T> = Result<T, FdbError>;

/// Error cases
#[derive(Debug, Error)]
pub enum FdbError {
    /// Stub marker for yet-to-be implemented features.
    #[error("unimplemented feature: {0}")]
    Unimplemented(&'static str),

    #[error("invalid process ID")]
    InvalidPid,

    #[error("system call failed: {0}")]
    SysCall(#[from] errno::Errno),

    #[error("C string error: {0}")]
    CStringError(#[from] NulError),
}
