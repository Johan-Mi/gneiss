#![deny(unsafe_code)]

mod ast;
mod compile;
mod lsp;
mod text;
mod typ;

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
    /// Run the language server
    Lsp(LspCommand),
}

#[derive(Options)]
struct CompileCommand {
    /// The source file to compile
    #[options(free, required)]
    file: PathBuf,
}

#[derive(Options)]
struct LspCommand {}

fn main() -> ExitCode {
    let opts = Opts::parse_args_default_or_exit();
    let Some(command) = opts.command else {
        println!("No command provided");
        if let Some(commands) = Opts::command_list() {
            println!("\nAvaliable commands:\n{commands}");
        }
        return ExitCode::FAILURE;
    };

    match command {
        Command::Compile(CompileCommand { file }) => compile::compile(&file),
        Command::Lsp(LspCommand {}) => {
            simplelog::WriteLogger::init(
                log::LevelFilter::Info,
                simplelog::Config::default(),
                std::fs::File::create("lsp.log").unwrap(),
            )
            .unwrap();
            std::panic::set_hook(Box::new(|panic_info| {
                log::error!("{panic_info}");
                log::error!("{}", std::backtrace::Backtrace::force_capture());
            }));

            lsp::LanguageServer::new().run();
        }
    }

    ExitCode::SUCCESS
}
