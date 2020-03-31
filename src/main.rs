#![allow(clippy::match_bool, clippy::fn_to_numeric_cast)]
#![feature(proc_macro_hygiene)]
extern crate dynasm;

use std::fs;

use structopt::StructOpt;

mod compiler;

#[derive(StructOpt)]
#[structopt(name = "gbjit")]
#[structopt(about = r#"
A WIP just-in-time compiler for the GameBoy and GameBoy Colour.

Currently just disassembles a given binary.
"#)]
struct Args {
    /// File to disassemble.
    binary: String,

    /// Whether to print just the commands or the full instructions
    #[structopt(short, long)]
    full: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();

    let data = fs::read(args.binary)?;

    let insts = compiler::decode(data.as_slice());

    let mut idx = 0;
    while idx < insts.len() {
        let i = &insts[idx];
        if args.full {
            println!("{:#05x}: {:?}", idx, i);
        } else {
            println!("{:#05x}: {:?}", idx, i.cmd);
        }
        idx += i.size() as usize;
    }

    Ok(())
}
