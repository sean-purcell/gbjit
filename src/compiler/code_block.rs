use std::mem;

use dynasmrt::{AssemblyOffset, ExecutableBuffer};

use super::Instruction;

pub struct CodeBlock {
    base_addr: u16,
    buf: ExecutableBuffer,
    entry: extern "sysv64" fn(),
    offsets: Vec<AssemblyOffset>,
    instructions: Vec<Instruction>,
}

impl CodeBlock {
    pub(super) fn new(
        base_addr: u16,
        buf: ExecutableBuffer,
        entry: AssemblyOffset,
        offsets: Vec<AssemblyOffset>,
        instructions: Vec<Instruction>,
    ) -> Self {
        let entry = unsafe { mem::transmute(buf.ptr(entry)) };
        CodeBlock {
            base_addr,
            buf,
            entry,
            offsets,
            instructions,
        }
    }

    pub fn instructions(&self) -> &[Instruction] {
        self.instructions.as_slice()
    }

    // TODO: Make cpu state and memory a parameter
    pub fn enter(&self) {
        (self.entry)()
    }
}
