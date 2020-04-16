#![allow(invalid_value)]

use std::collections::HashSet;
use std::convert::TryInto;
use std::mem;

use gbjit::compiler::{codegen, decoder, CompileOptions, ExternalBus, Instruction};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let insts: HashSet<Instruction> = (0u32..(1 << 24))
        .map(|idx| {
            let bytes = idx.to_le_bytes();
            decoder::decode_full(bytes[0..3].try_into().unwrap())
        })
        .collect();

    println!("{} total distinct instructions", insts.len());

    let insts: Vec<Instruction> = insts.iter().cloned().collect();

    let bus = unsafe {
        ExternalBus::<()> {
            read: mem::transmute(0usize),
            write: mem::transmute(0usize),
            interrupts: mem::transmute(0usize),
        }
        .type_erased()
    };

    let options = CompileOptions { trace_pc: false };

    let blocks: Vec<(Vec<Instruction>, Vec<usize>)> = insts
        .chunks(32768)
        .map(|chunk| {
            let mut chunk = chunk.to_vec();
            // add one more at the end to compare offsets against
            chunk.push(Instruction::invalid(0));

            let offsets = codegen::codegen(0, chunk.as_slice(), &bus, &options)
                .unwrap()
                .2
                .iter()
                .map(|x| x.0)
                .collect();
            chunk.pop();
            (chunk, offsets)
        })
        .collect();

    println!("Compiled {} blocks", blocks.len());

    let lengths: Vec<(Instruction, usize)> = blocks
        .iter()
        .flat_map(|(insts, offsets)| {
            insts[0..insts.len() - 1]
                .iter()
                .zip(offsets.windows(2))
                .map(|(inst, offsets)| (inst.clone(), offsets[1] - offsets[0]))
        })
        .collect();

    let min = lengths.iter().min_by_key(|(_, len)| len).unwrap();
    let max = lengths.iter().max_by_key(|(_, len)| len).unwrap();

    println!("Shorters instruction: {:?}", min);
    println!("Longest instruction: {:?}", max);

    Ok(())
}
