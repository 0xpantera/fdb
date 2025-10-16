//! Process management wrapping `ptrace` interactions.

use std::ffi::CString;

use nix::{sys::ptrace, unistd::Pid};
use nix::{
    sys::wait::waitpid,
    unistd::{ForkResult, execvp, fork},
};

use crate::errors::{FdbError, FdbResult};

/// Represents a traced process under the debugger's control.
#[derive(Debug)]
pub struct ProcessHandle {
    pid: Pid,
}

impl ProcessHandle {
    /// Launch a new debugger process.
    pub fn launch(_program: &str, _args: &[String]) -> FdbResult<Self> {
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                waitpid(child, None)?;
                Ok(ProcessHandle { pid: child })
            }
            Ok(ForkResult::Child) => {
                // Child process - handle errors by exiting, never return
                if let Err(e) = ptrace::traceme() {
                    // let parent debug us
                    eprintln!("Failed to set up tracing: {}", e);
                    std::process::exit(1);
                }

                let program = match CString::new(_program) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Invalid program name: {}", e);
                        std::process::exit(1);
                    }
                };

                let args: Vec<CString> =
                    match _args.iter().map(|s| CString::new(s.as_str())).collect() {
                        Ok(args) => args,
                        Err(e) => {
                            eprintln!("Invalid arguments: {}", e);
                            std::process::exit(1);
                        }
                    };

                // Replace this process with target program
                execvp(&program, &args).unwrap_or_else(|e| {
                    eprintln!("Failed to exec {}: {}", _program, e);
                    std::process::exit(1);
                });
                // execvp succeeded - process is replaced, this never executes
                unreachable!("execvp should replace the process")
            }
            Err(e) => Err(FdbError::SysCall(e)),
        }
    }

    /// Attach to an existing PID using `ptrace`.
    pub fn attach(pid: i32) -> FdbResult<Self> {
        let _pid = Pid::from_raw(pid);
        if _pid.as_raw() <= 0 {
            return Err(FdbError::InvalidPid);
        }

        ptrace::attach(_pid)?;
        Ok(ProcessHandle { pid: _pid })
    }

    /// Access the underlying PID for logging or future syscalls.
    pub fn pid(&self) -> Pid {
        self.pid
    }
}
