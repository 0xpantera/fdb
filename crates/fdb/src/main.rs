//! Command-line interface for the fdb debugger.

use anyhow::Result;
use clap::{Parser, Subcommand};
use libfdb::process::ProcessHandle;
use log::info;

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
        Command::Run { prog, args } => run_program(&prog, &args),
        Command::Attach { pid } => attach_to_process(pid),
        Command::Version => {
            println!("fdb {}", libfdb::version());
            Ok(())
        }
    }
}

fn run_program(prog: &str, args: &[String]) -> Result<()> {
    info!("Launching {prog} with args {args:?}");
    let _handle = ProcessHandle::launch(prog, args)?;
    Ok(())
}

fn attach_to_process(pid: i32) -> Result<()> {
    info!("Attaching to pid {pid}");
    let _handle = ProcessHandle::attach(pid)?;
    Ok(())
}
