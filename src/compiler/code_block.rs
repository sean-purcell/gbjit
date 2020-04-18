use std::cmp::Ordering;
use std::ffi::c_void;
use std::fmt;
use std::mem;

use capstone::Capstone;
use capstone::Error as CsError;
use dynasmrt::{AssemblyOffset, ExecutableBuffer};

use crate::cpu_state::CpuState;

use super::external_bus::Wrapper as BusWrapper;
use super::{ExternalBus, Instruction};

pub struct CodeBlock<T> {
    base_addr: u16,
    buf: ExecutableBuffer,
    entry: extern "sysv64" fn(*mut CpuState, target_pc: u64, param: *mut c_void),
    offsets: Vec<AssemblyOffset>,
    instructions: Vec<Result<Instruction, Vec<u8>>>,
    bus: ExternalBus<T>,
}

impl<T> CodeBlock<T> {
    pub(super) fn new(
        base_addr: u16,
        buf: ExecutableBuffer,
        entry: AssemblyOffset,
        offsets: Vec<AssemblyOffset>,
        instructions: Vec<Result<Instruction, Vec<u8>>>,
        bus: ExternalBus<T>,
    ) -> Self {
        let entry = unsafe { mem::transmute(buf.ptr(entry)) };
        CodeBlock {
            base_addr,
            buf,
            entry,
            offsets,
            instructions,
            bus,
        }
    }

    pub fn instructions(&self) -> &[Result<Instruction, Vec<u8>>] {
        self.instructions.as_slice()
    }

    pub fn enter(&self, cpu_state: &mut CpuState, param: &mut T) {
        let gb_pc = cpu_state.pc;
        let len = self.offsets.len();
        assert!(
            gb_pc >= self.base_addr && gb_pc - self.base_addr < len as u16,
            "PC not within appropriate range: pc: {}, base_addr: {}, len: {}",
            gb_pc,
            self.base_addr,
            len
        );
        let target_pc = self.buf.ptr(self.offsets[gb_pc as usize]);

        let mut wrapper = BusWrapper::new(&self.bus, param);
        let void_wrapper = unsafe { mem::transmute(&mut wrapper as *mut BusWrapper<T>) };

        (self.entry)(cpu_state as *mut CpuState, target_pc as u64, void_wrapper)
    }

    pub fn disassemble(&self) -> Result<Vec<String>, CsError> {
        use capstone::arch::x86;
        use capstone::arch::{BuildsCapstone, BuildsCapstoneSyntax};

        let cs = Capstone::new()
            .x86()
            .mode(x86::ArchMode::Mode64)
            .syntax(x86::ArchSyntax::Intel)
            .detail(false)
            .build()?;

        let base_addr = self.buf.ptr(AssemblyOffset(0)) as u64;

        let instructions = cs.disasm_all(&*self.buf, base_addr)?;

        enum Entry<'a> {
            SrcInstruction {
                src_pc: u16,
                host_pc: u64,
                inst: &'a Result<Instruction, Vec<u8>>,
            },
            HostInstruction {
                host_pc: u64,
                repr: String,
            },
        }

        impl<'a> Entry<'a> {
            fn sort_idx(&self) -> u64 {
                use Entry::*;
                match self {
                    SrcInstruction {
                        src_pc: _,
                        host_pc,
                        inst: _,
                    } => 2 * *host_pc,
                    HostInstruction { host_pc, repr: _ } => (2 * *host_pc) + 1,
                }
            }
        }

        impl<'a> fmt::Display for Entry<'a> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                use Entry::*;
                match self {
                    SrcInstruction {
                        src_pc,
                        host_pc: _,
                        inst,
                    } => {
                        write!(f, "{:#06x?}: ", src_pc)?;
                        match inst {
                            Ok(i) => write!(f, "{:?}", i.cmd),
                            Err(bytes) => write!(f, "{:02x?}", bytes),
                        }
                    }
                    HostInstruction { host_pc: _, repr } => write!(f, "{}", repr),
                }
            }
        }

        impl<'a> PartialEq for Entry<'a> {
            fn eq(&self, other: &Entry<'a>) -> bool {
                self.sort_idx() == other.sort_idx()
            }
        }

        impl<'a> PartialOrd for Entry<'a> {
            fn partial_cmp(&self, other: &Entry<'a>) -> Option<Ordering> {
                Some(self.sort_idx().cmp(&other.sort_idx()))
            }
        }

        let src_insts = self
            .offsets
            .iter()
            .enumerate()
            .map(|(i, o)| Entry::SrcInstruction {
                src_pc: self.base_addr + i as u16,
                host_pc: self.buf.ptr(*o) as u64,
                inst: &self.instructions[i],
            });
        let host_insts = instructions.iter().map(|x| Entry::HostInstruction {
            host_pc: x.address(),
            repr: x.to_string(),
        });

        Ok(itertools::merge(src_insts, host_insts)
            .map(|x| x.to_string())
            .collect())
    }
}
