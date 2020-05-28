#![allow(
    clippy::match_bool, // Code reads cleaner this way sometimes
    clippy::fn_to_numeric_cast, // Necessary for dynasm
    clippy::transmute_ptr_to_ptr // Makes code cleaner when the destination type is clear
)]
#![feature(proc_macro_hygiene)]

use structopt::StructOpt;

mod args;
mod compiler;
mod cpu_state;
mod executor;
mod frontend;
mod gb;

use args::Args;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::from_args();

    if args.headless {
        frontend::headless::run(args)
    } else {
        frontend::gui::run(args)
    }
}
