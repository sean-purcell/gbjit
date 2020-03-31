#![allow(dead_code)]

use std::fmt;

use dynasmrt::AssemblyOffset;
use dynasmrt::ExecutableBuffer;

mod decoder;
pub mod instruction;

pub use instruction::Instruction;

/// The number of instructions assembled for a block
pub const INSTRUCTIONS_PER_BLOCK: usize = 256;

pub struct CodeBlock {
    base_addr: u16,
    buf: ExecutableBuffer,
    entry: AssemblyOffset,
    offsets: [AssemblyOffset; INSTRUCTIONS_PER_BLOCK],
}

#[derive(Debug, Copy, Clone)]
pub enum CompileError {
    FailedRead,
    UnalignedBase,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CompileError {}

pub fn compile(
    base_addr: u16,
    _read: impl Fn(u16) -> Option<u8>,
) -> Result<CodeBlock, CompileError> {
    if base_addr & ((INSTRUCTIONS_PER_BLOCK - 1) as u16) != 0 {
        return Err(CompileError::UnalignedBase);
    }

    unimplemented!()
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
