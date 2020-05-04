use std::{fmt, io, iter};

use dynasmrt::DynasmError;

mod code_block;
pub mod codegen;
mod cycle_state;
pub mod decoder;
mod external_bus;
pub mod instruction;
mod oneoff_table;

pub use code_block::CodeBlock;

pub use cycle_state::CycleState;

pub use external_bus::Generic as ExternalBus;

pub use instruction::Instruction;

pub use oneoff_table::OneoffTable;

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
    bytes: &[u8],
    bus: ExternalBus<T>,
    oneoffs: &OneoffTable,
    options: &CompileOptions,
) -> Result<CodeBlock<T>, CompileError> {
    let none_if_empty: for<'a> fn(&'a [u8]) -> Option<&'a [u8]> =
        |b: &[u8]| if b.is_empty() { None } else { Some(b) };
    let instructions: Vec<Result<Instruction, Vec<u8>>> =
        iter::successors(none_if_empty(bytes), |prev| none_if_empty(&prev[1..]))
            .map(|bytes| {
                let req = decoder::bytes_required(bytes[0]) as usize;
                if req > bytes.len() {
                    Err(bytes.to_vec())
                } else {
                    Ok(decoder::decode(&bytes[0..req]).expect("Byte count should be correct"))
                }
            })
            .collect();

    let (buf, entry, offsets) = codegen::codegen_block(
        base_addr,
        instructions.as_slice(),
        &bus.type_erased(),
        oneoffs,
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
