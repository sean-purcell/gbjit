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

    /// Whether to print the disassembled rom
    #[structopt(short, long)]
    disassemble: bool,

    /// Whether to print just the commands or the full instructions
    #[structopt(short, long)]
    full_disassembly: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();

    let data = fs::read(args.binary)?;

    let block = compiler::compile(0, data.len() as u16, |x| data.get(x as usize).map(|x| *x))?;

    if args.disassemble {
        print_disassembly(&block, args.full_disassembly);
    }

    block.enter();

    Ok(())
}

fn print_disassembly(block: &compiler::CodeBlock, full: bool) {
    let insts = block.instructions();
    let mut idx = 0;
    while idx < insts.len() {
        let i = &insts[idx];
        if full {
            println!("{:#05x}: {:?}", idx, i);
        } else {
            println!("{:#05x}: {:?}", idx, i.cmd);
        }
        idx += i.size() as usize;
    }
}
