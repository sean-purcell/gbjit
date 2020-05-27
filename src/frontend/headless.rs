use std::error::Error as StdError;

use log::*;

use crate::{executor::ExecutorOptions, gb::Gb, Args};

pub fn run(args: Args) -> Result<(), Box<dyn StdError>> {
    let mut gb = Gb::new(
        &args.bios,
        &args.rom,
        ExecutorOptions {
            trace_pc: args.trace_pc,
            disassembly_logfile: args.disassembly_logfile.clone(),
        },
    )?;

    let mut i = 0;
    loop {
        gb.run_frame()?;
        debug!("Finished frame {}", i);
        i += 1;
    }
}
