//! Process management scaffolding wrapping `ptrace` interactions.

use nix::unistd::Pid;

use crate::errors::{FdbError, FdbResult};

/// Represents a traced process under the debugger's control.
#[derive(Debug)]
pub struct ProcessHandle {
    pid: Pid,
}

impl ProcessHandle {
    /// Launch a new debugger process.
    pub fn launch(_program: &str, _args: &[String]) -> FdbResult<Self> {
        Err(FdbError::Unimplemented("process::launch"))
    }

    /// Attach to an existing PID using `ptrace`.
    pub fn attach(pid: i32) -> FdbResult<Self> {
        let _pid = Pid::from_raw(pid);
        Err(FdbError::Unimplemented("process::attach"))
    }

    /// Access the underlying PID for logging or future syscalls.
    pub fn pid(&self) -> Pid {
        self.pid
    }
}
