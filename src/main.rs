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

use gb::bus::{Bus, BusWrapper};
use gb::devices::Ppu;

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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::from_args();

    frontend::gui::run(&args)
}

fn print_disassembly<T>(block: &compiler::CodeBlock<T>, full: bool) {
    let insts = block.instructions();
    let mut idx = 0;
    while idx < insts.len() {
        let i = &insts[idx];
        if full {
            println!("{:#05x}: {:?}", idx, i);
        } else {
            println!("{:#05x?}: ", idx);
            match i {
                Ok(i) => println!("{:?}", i.cmd),
                Err(bytes) => println!("{:02x?}", bytes),
            }
        }
        idx += match i {
            Ok(i) => i.size() as usize,
            Err(bytes) => bytes.len(),
        };
    }
}
