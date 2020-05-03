use dynasmrt::{AssemblyOffset, ExecutableBuffer};
use rayon::prelude::*;

use super::external_bus::{Generic, TypeErased};
use super::{codegen, decoder, CompileError, CompileOptions};

pub struct SingleTable(ExecutableBuffer, AssemblyOffset);

pub struct OneoffTable(Vec<SingleTable>);

impl SingleTable {
    fn generate(
        first_byte: u8,
        bus: &TypeErased,
        options: &CompileOptions,
    ) -> Result<Self, CompileError> {
        let req = decoder::bytes_required(first_byte);
        let upper_bound = 1u32 << ((req - 1) * 8);
        let instructions: Vec<_> = (0..upper_bound)
            .map(|idx| {
                let bytes = [first_byte, idx as u8, (idx >> 8) as u8];
                decoder::decode_full(bytes)
            })
            .collect();

        let (buf, offset) = codegen::codegen_oneoffs(instructions.as_slice(), bus, options)?;

        Ok(SingleTable(buf, offset))
    }

    pub fn base(&self) -> *const u8 {
        self.0.ptr(AssemblyOffset(0))
    }

    pub fn table(&self) -> *const u8 {
        self.0.ptr(self.1)
    }
}

impl OneoffTable {
    pub fn generate_raw(bus: &TypeErased, options: &CompileOptions) -> Result<Self, CompileError> {
        let tables = (0u8..=255u8)
            .into_par_iter()
            .map(|byte| SingleTable::generate(byte, bus, options))
            .collect::<Result<Vec<SingleTable>, CompileError>>()?;

        Ok(OneoffTable(tables))
    }

    pub fn generate<T>(bus: &Generic<T>, options: &CompileOptions) -> Result<Self, CompileError> {
        Self::generate_raw(&bus.type_erased(), options)
    }

    pub fn get_table(&self, first_byte: u8) -> &SingleTable {
        &self.0[first_byte as usize]
    }
}
