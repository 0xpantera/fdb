//! Command-line interface for the fdb debugger.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use libfdb::process::ProcessHandle;
use log::info;
use nix::{
    sys::{
        ptrace,
        wait::{self, WaitPidFlag, WaitStatus},
    },
    unistd::Pid,
};
use rustyline::{Editor, error::ReadlineError, history::DefaultHistory};

/// Top-level argument parser describing the debugger interface.
#[derive(Parser, Debug)]
#[command(name = "fdb", version, about = "A simple Linux debugger in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

/// Subcommands exposed by the debugger frontend.
#[derive(Subcommand, Debug)]
enum Command {
    /// Run a program under fdb control.
    Run {
        prog: String,
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Attach to an existing process ID.
    Attach { pid: i32 },
    /// Display version information for diagnostics.
    Version,
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    match cli.cmd {
        Command::Run { prog, args } => {
            let process = run_program(&prog, &args)?;
            run_interactive_session(process)?;
        }
        Command::Attach { pid } => {
            let process = attach_to_process(pid)?;
            run_interactive_session(process)?;
        }
        Command::Version => {
            println!("fdb {}", libfdb::version());
        }
    };
    Ok(())
}

fn run_program(prog: &str, args: &[String]) -> Result<ProcessHandle> {
    info!("Launching {prog} with args {args:?}");
    let handle = ProcessHandle::launch(prog, args)
        .with_context(|| format!("Failed to launch program '{}'", prog))?;
    Ok(handle)
}

fn attach_to_process(pid: i32) -> Result<ProcessHandle> {
    info!("Attaching to pid {pid}");
    let handle = ProcessHandle::attach(pid)
        .with_context(|| format!("Failed to attach to process {}", pid))?;
    Ok(handle)
}

fn run_interactive_session(process: ProcessHandle) -> Result<()> {
    println!(
        "Attached to process {} - entering interactive mode",
        process.pid()
    );

    let mut rl = Editor::<(), DefaultHistory>::new()?;

    // Optional: persistent history (similar spirit to libedit’s history)
    // don't want a new dep to locate $HOME, just use a local file.
    let hist_file = ".fdb_history";
    let _ = rl.load_history(hist_file);

    // Track the last non-empty command, to re-run on empty input
    let mut last_cmd: Option<String> = None;

    loop {
        match rl.readline("fdb> ") {
            Ok(line) => {
                let trimmed = line.trim();

                // If empty: rerun last command (if any), don't add to history
                let to_run = if trimmed.is_empty() {
                    match &last_cmd {
                        Some(prev) => prev.as_str(),
                        None => {
                            // Nothing to repeat; behave like no-op
                            continue;
                        }
                    }
                } else {
                    // Non-empty: update last_cmd and add to history, like libedit code
                    rl.add_history_entry(trimmed)?;
                    last_cmd = Some(trimmed.to_string());
                    trimmed
                };

                // Handle command; exit handled in loop after call
                if let Err(e) = handle_command(&process, to_run) {
                    eprintln!("Error: {e}");
                }

                // Quit commands
                if matches!(to_run, "quit" | "exit") {
                    break;
                }
            }

            Err(ReadlineError::Interrupted) => {
                // Ctrl-C: match shell debuggers (print caret and continue)
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D: end-of-file → exit loop
                println!("^D");
                break;
            }
            Err(err) => {
                eprintln!("Error reading line: {err}");
                break;
            }
        }
    }

    // Try to persist history (ignore errors)
    let _ = rl.save_history(hist_file);

    println!("Exiting debugger");
    Ok(())
}

fn handle_command(process: &ProcessHandle, line: &str) -> Result<()> {
    let args = split_whitespace(line);
    if args.is_empty() {
        return Ok(());
    }

    let cmd = args[0];

    // Accept "c", "cont", "continue" (prefix match on "continue")
    if is_prefix(cmd, "continue") {
        let pid = process.pid();
        resume(pid)?;
        // after continue immediately wait for the next stop.
        wait_on_signal(pid)?;
        return Ok(());
    }

    match cmd {
        "help" => {
            println!("Available commands:");
            println!("  help              - show this help");
            println!("  continue|cont|c   - resume the program");
            println!("  info              - show process info");
            println!("  quit|exit         - exit debugger");
        }
        "info" => {
            println!("Process PID: {}", process.pid());
        }
        "quit" | "exit" => {
            // handled by the outer loop
        }
        _ => {
            eprintln!("Unknown command");
        }
    }
    Ok(())
}

fn split_whitespace(line: &str) -> Vec<&str> {
    // GDB/LLDB-style CLIs generally treat any whitespace as a separator.
    line.split_whitespace().collect()
}

fn is_prefix<S: AsRef<str>>(s: S, of: S) -> bool {
    let s = s.as_ref();
    let of = of.as_ref();
    of.starts_with(s)
}

fn resume(pid: Pid) -> anyhow::Result<()> {
    // ptrace(PTRACE_CONT, pid, nullptr, nullptr)
    ptrace::cont(pid, None).map_err(|e| anyhow::anyhow!("Couldn't continue: {e}"))?;
    Ok(())
}

fn wait_on_signal(pid: Pid) -> anyhow::Result<()> {
    // waitpid(pid, &status, 0)
    match wait::waitpid(pid, Some(WaitPidFlag::empty()))? {
        WaitStatus::Stopped(_, sig) => {
            // Tracee stopped on a signal (expected after CONT if a breakpoint/signal hits)
            // You can print or log the signal here if you like.
            log::debug!("stopped by signal: {sig}");
        }
        WaitStatus::Exited(_, code) => {
            anyhow::bail!("process exited with code {code}");
        }
        WaitStatus::Signaled(_, sig, _core) => {
            anyhow::bail!("process terminated by signal {sig}");
        }
        other => {
            // Covers Continued/StillAlive/PtraceEvent/PtraceSyscall, etc.
            log::debug!("waitpid: {:?}", other);
        }
    }
    Ok(())
}
