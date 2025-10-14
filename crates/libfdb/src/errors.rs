//! Error taxonomy placeholder for the debugger core.

use thiserror::Error;

/// Unified result type across the debugger core.
pub type FdbResult<T> = Result<T, FdbError>;

/// Error cases that will be refined once functionality lands.
#[derive(Debug, Error)]
pub enum FdbError {
    /// Stub marker for yet-to-be implemented features.
    #[error("unimplemented feature: {0}")]
    Unimplemented(&'static str),

    /// Wrapper for low-level syscall failures; expands as integration progresses.
    #[error("system error: {0}")]
    System(&'static str),
}
