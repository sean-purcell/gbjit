#![allow(clippy::match_bool, clippy::fn_to_numeric_cast)]
#![feature(proc_macro_hygiene)]
extern crate dynasm;

use std::fs;

use log::*;
use structopt::StructOpt;

mod compiler;
mod cpu_state;

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

    /// Whether to print the disassembly of the code block
    #[structopt(short, long)]
    x64_disasm: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::from_args();

    let data = fs::read(args.binary)?;

    let block = compiler::compile(
        0,
        data.len() as u16,
        |x| data.get(x as usize).copied(),
        compiler::ExternalBus {
            read: dummy_read,
            write: dummy_write,
            interrupts: dummy_interrupts,
        },
    )?;

    if args.disassemble {
        print_disassembly(&block, args.full_disassembly);
    }

    if args.x64_disasm {
        println!("Disassembly:");
        for i in block.disassemble()? {
            println!("{}", i);
        }
        println!();
    }

    let mut cpu_state = cpu_state::CpuState::new();

    block.enter(&mut cpu_state, &mut ());

    println!("{:?}", cpu_state);

    Ok(())
}

fn print_disassembly<T>(block: &compiler::CodeBlock<T>, full: bool) {
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

fn dummy_read(_: &mut (), addr: u16) -> (bool, u8) {
    debug!("Read  {:#06x?} -> {:#04x}", addr, 0);
    (false, 0)
}

fn dummy_write(_: &mut (), addr: u16, val: u8) -> bool {
    debug!("Write {:#06x?} <- {:#04x}", addr, val);
    false
}

fn dummy_interrupts(_: &mut (), enabled: bool) -> bool {
    debug!("Interrupts enabled: {}", enabled);
    false
}
