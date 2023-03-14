#![deny(unsafe_code)]

use gumdrop::Options;

#[derive(Options)]
struct Opts {
    #[options(help = "Print help information")]
    help: bool,
}

fn main() {
    let opts = Opts::parse_args_default_or_exit();
}
