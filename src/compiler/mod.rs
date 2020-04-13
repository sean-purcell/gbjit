use std::convert::TryInto;
use std::fmt;
use std::io;

use dynasmrt::DynasmError;

mod code_block;
mod codegen;
mod decoder;
mod external_bus;
pub mod instruction;

pub use code_block::CodeBlock;

pub use external_bus::Generic as ExternalBus;

pub use instruction::Instruction;

#[derive(Debug)]
pub enum CompileError {
    IoError(io::Error),
    DynasmError(DynasmError),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CompileError {}

impl From<io::Error> for CompileError {
    fn from(e: io::Error) -> Self {
        CompileError::IoError(e)
    }
}

impl From<DynasmError> for CompileError {
    fn from(e: DynasmError) -> Self {
        CompileError::DynasmError(e)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CompileOptions {
    pub trace_pc: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        CompileOptions { trace_pc: false }
    }
}

pub fn compile<T>(
    base_addr: u16,
    len: u16,
    read: impl Fn(u16) -> Option<u8>,
    bus: ExternalBus<T>,
    options: &CompileOptions,
) -> Result<CodeBlock<T>, CompileError> {
    let padded: Box<[u8]> = (0..len + 2).map(read).map(|x| x.unwrap_or(0)).collect();
    let instructions: Vec<Instruction> = padded
        .windows(3)
        .map(|bytes| decoder::decode_full(bytes.try_into().unwrap()))
        .collect();

    let (buf, entry, offsets) = codegen::codegen(
        base_addr,
        instructions.as_slice(),
        &bus.type_erased(),
        options,
    )?;

    Ok(CodeBlock::new(
        base_addr,
        buf,
        entry,
        offsets,
        instructions,
        bus,
    ))
}

#[allow(dead_code)]
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
