#![allow(
    clippy::match_bool, // Code reads cleaner this way sometimes
    clippy::fn_to_numeric_cast, // Necessary for dynasm
    clippy::transmute_ptr_to_ptr // Makes code cleaner when the destination type is clear
)]
#![feature(proc_macro_hygiene)]

extern crate dynasm;

use structopt::StructOpt;

mod compiler;
mod cpu_state;
mod executor;
mod frontend;
mod gb;

#[derive(StructOpt)]
#[structopt(name = "gbjit")]
#[structopt(about = r#"
A WIP just-in-time compiler for the GameBoy and GameBoy Colour.

Currently just disassembles a given binary.
"#)]
pub struct Args {
    /// GB bios file
    bios: String,

    /// GB rom to run
    rom: String,

    /// Logfile to write GB and x86 disassembly to
    #[structopt(short, long)]
    disassembly_logfile: Option<String>,

    /// Whether to generate log traces for each instruction executed
    #[structopt(short, long)]
    trace_pc: bool,

    #[structopt(
        short = "p",
        long = "px",
        default_value = "960,864",
        parse(try_from_str = parse_tuple)
    )]
    screen_dimensions: (u32, u32),

    /// Only advance the frame when the 'n' key is hit
    #[structopt(short, long)]
    wait: bool,

    /// Whether to run in headless mode, where the gb is emulated with no IO, just to generate logs
    #[structopt(short, long)]
    headless: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::from_args();

    if args.headless {
        frontend::headless::run(args)
    } else {
        frontend::gui::run(args)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Failed to parse {src}")]
struct DimensionParseError {
    src: String,
}

impl From<&str> for DimensionParseError {
    fn from(s: &str) -> Self {
        DimensionParseError {
            src: String::from(s),
        }
    }
}

fn parse_tuple(src: &str) -> Result<(u32, u32), DimensionParseError> {
    use std::str::FromStr;

    let components: Result<Vec<u32>, std::num::ParseIntError> =
        src.split(",").map(u32::from_str).collect();
    let components = components.map_err(|_| DimensionParseError::from(src))?;
    match *components {
        [w, h] => Ok((w, h)),
        _ => Err(src.into()),
    }
}
