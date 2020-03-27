use dynasmrt::AssemblyOffset;
use dynasmrt::ExecutableBuffer;

/// The number of instructions assembled for a block
pub const INSTRUCTIONS_PER_BLOCK: usize = 256;
/// The number of bytes required from memory to assemble a block
pub const DATA_BYTES_PER_BLOCK: usize = INSTRUCTIONS_PER_BLOCK + 2;

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

pub fn compile(
    base_addr: u16,
    read: impl Fn(u16) -> Option<u8>,
) -> Result<CodeBlock, CompileError> {
    if base_addr & ((INSTRUCTIONS_PER_BLOCK - 1) as u16) != 0 {
        return Err(CompileError::UnalignedBase);
    }

    unimplemented!()
}
