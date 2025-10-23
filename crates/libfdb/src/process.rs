//! Process management wrapping `ptrace` interactions.

use std::ffi::CStr;

use nix::{sys::ptrace, unistd::Pid};
use nix::{
    sys::wait::waitpid,
    unistd::{ForkResult, execvp, fork},
};

use crate::errors::{FdbError, FdbResult};
use crate::{ProcessState, StopReason};

/// Represents a traced process under the debugger's control.
#[derive(Debug)]
pub struct ProcessHandle {
    pid: Pid,
    state: ProcessState,
}

impl ProcessHandle {
    /// Launch a new debugger process.
    pub fn launch(program: &CStr, args: &[&CStr]) -> FdbResult<Self> {
        match unsafe { fork()? } {
            ForkResult::Parent { child, .. } => {
                waitpid(child, None)?;
                Ok(ProcessHandle {
                    pid: child,
                    state: ProcessState::Initialized,
                })
            }
            ForkResult::Child => {
                ptrace::traceme().unwrap_or_else(|e| {
                    eprintln!("TRACEME: {e}");
                    std::process::exit(1);
                });
                let _ = nix::sys::signal::raise(nix::sys::signal::Signal::SIGSTOP);
                execvp(&program, &args).unwrap_or_else(|e| {
                    eprintln!("execvp: {e}");
                    std::process::exit(1);
                });
                unreachable!()
            }
        }
    }

    /// Attach to an existing PID using `ptrace`.
    pub fn attach(pid: i32) -> FdbResult<Self> {
        let _pid = Pid::from_raw(pid);
        if _pid.as_raw() <= 0 {
            return Err(FdbError::InvalidPid);
        }

        ptrace::attach(_pid)?;
        waitpid(_pid, None)?;
        Ok(ProcessHandle {
            pid: _pid,
            state: ProcessState::Initialized,
        })
    }

    pub fn wait_on_signal(&mut self) -> FdbResult<ProcessState> {
        use nix::sys::wait::WaitStatus::*;
        match waitpid(self.pid, None)? {
            Stopped(_, sig) => {
                let sig = sig; // nix::sys::signal::Signal
                let reason = StopReason { signal: sig };
                self.state = ProcessState::Stopped(reason);
                Ok(self.state)
            }
            Exited(_, code) => {
                self.state = ProcessState::Exited(code);
                Ok(self.state)
            }
            Signaled(_, sig, _core) => {
                self.state = ProcessState::Terminated(sig);
                Ok(self.state)
            }
            other => {
                // We shouldn’t see others much; surface as Stopped(SIGTRAP)-ish later.
                // For now just keep it simple and treat as a generic stop if it happens.
                if let Stopped(_, sig) = other {
                    let reason = StopReason { signal: sig };
                    self.state = ProcessState::Stopped(reason);
                    Ok(self.state)
                } else {
                    // fallback: don't change state; return current
                    Ok(self.state)
                }
            }
        }
    }

    pub fn state(&self) -> ProcessState {
        self.state
    }

    pub fn resume(&mut self) -> FdbResult<()> {
        ptrace::cont(self.pid, None)?;
        self.state = ProcessState::Running;
        Ok(())
    }

    /// Access the underlying PID for logging or future syscalls.
    pub fn pid(&self) -> Pid {
        self.pid
    }
}
