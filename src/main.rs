#![deny(unsafe_code)]

mod compile;

use gumdrop::Options;
use std::{path::PathBuf, process::ExitCode};

#[derive(Options)]
struct Opts {
    #[options(help = "Print help information")]
    help: bool,

    #[options(command)]
    command: Option<Command>,
}

#[derive(Options)]
enum Command {
    /// Compile a source file into an executable
    Compile(CompileCommand),
}

#[derive(Options)]
struct CompileCommand {
    /// The source file to compile
    #[options(free)]
    file: PathBuf,
}

fn main() -> ExitCode {
    let opts = Opts::parse_args_default_or_exit();
    match opts.command {
        Some(Command::Compile(CompileCommand { file })) => {
            compile::compile(&file);
        }
        None => {
            println!("No command provided");
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
