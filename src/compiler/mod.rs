#![allow(dead_code)]

use std::convert::TryInto;
use std::fmt;

mod code_block;
mod codegen;
mod decoder;
pub mod instruction;

pub use code_block::CodeBlock;

pub use instruction::Instruction;

#[derive(Debug, Copy, Clone)]
pub enum CompileError {}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CompileError {}

pub fn compile(
    base_addr: u16,
    len: u16,
    read: impl Fn(u16) -> Option<u8>,
) -> Result<CodeBlock, Box<dyn std::error::Error>> {
    let padded: Box<[u8]> = (0..len + 2).map(read).map(|x| x.unwrap_or(0)).collect();
    let instructions: Box<[Instruction]> = padded
        .windows(3)
        .map(|bytes| decoder::decode_full(bytes.try_into().unwrap()))
        .collect();

    codegen::codegen(base_addr, &*instructions)
}

pub fn decode(data: &[u8]) -> Vec<Instruction> {
    let mut padded = data.to_vec();
    padded.extend([0, 0].iter()); // Pad the data a bit in case the last instruction is long
    data.iter()
        .enumerate()
        .map(|(i, b)| {
            let req = decoder::bytes_required(*b);

            // TODO: format! in expect is moderately expensive
            decoder::decode(&padded[i..i + req as usize]).expect(&*format!(
                "Decode error should be impossible, byte: {:#04x?}, length: {}",
                b, req,
            ))
        })
        .collect()
}
