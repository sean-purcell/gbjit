use std::ffi::c_void;

use dynasm::dynasm;
use dynasmrt::x64::Assembler;
use dynasmrt::{AssemblyOffset, DynamicLabel, DynasmApi, DynasmLabelApi, ExecutableBuffer};
use log::*;

use crate::cpu_state::CpuState;

use super::external_bus::TypeErased as ExternalBus;
use super::instruction::{self, *};
use super::{decoder, CompileError, CompileOptions, OneoffTable};

#[macro_use]
mod util;

mod addhl;
mod addsp;
mod aluhalf;
mod bithalf;
mod call;
mod control;
mod cpl;
mod daa;
mod hlspoffset;
mod incdecfull;
mod incdechalf;
mod jump;
mod ldaddrinc;
mod ldfullimm;
mod ldhalf;
mod ldsphl;
mod pop;
mod push;
mod ret;
mod rst;
mod storesp;

use util::{
    call_read, pop_state, push_state, repack_cpu_state, setup_limit_address, unpack_cpu_state,
};

pub fn codegen_block(
    base_addr: u16,
    insts: &[Result<Instruction, Vec<u8>>],
    bus: &ExternalBus,
    oneoffs: &OneoffTable,
    options: &CompileOptions,
) -> Result<(ExecutableBuffer, AssemblyOffset, Vec<AssemblyOffset>), CompileError> {
    let mut ops = Assembler::new()?;

    let entry = generate_boilerplate(&mut ops);

    let labels: Vec<DynamicLabel> = insts.iter().map(|_| ops.new_dynamic_label()).collect();

    let offsets = insts
        .iter()
        .zip(labels.iter())
        .enumerate()
        .map(|(idx, (inst, label))| {
            let pc = base_addr.wrapping_add(idx as u16);
            match inst {
                Ok(i) => assemble_instruction(
                    &mut ops,
                    i,
                    label,
                    AssemblyKind::Static {
                        base_addr,
                        pc,
                        labels: &labels,
                    },
                    bus,
                    options.trace_pc,
                ),
                Err(bytes) => {
                    assemble_incomplete(&mut ops, bytes.as_slice(), label, pc, bus, oneoffs)
                }
            }
        })
        .collect();

    generate_overrun(&mut ops);

    generate_dynamic_jump_table(&mut ops, base_addr, labels.as_slice());

    ops.commit()
        .expect("No assembly errors should have occurred");

    let buf = ops.finalize().expect("No executor instances created");

    Ok((buf, entry, offsets))
}

pub fn codegen_oneoffs(
    insts: &[Instruction],
    bus: &ExternalBus,
    options: &CompileOptions,
) -> Result<(ExecutableBuffer, AssemblyOffset), CompileError> {
    let mut ops = Assembler::new()?;

    let labels: Vec<DynamicLabel> = insts.iter().map(|_| ops.new_dynamic_label()).collect();

    insts.iter().zip(labels.iter()).for_each(|(inst, label)| {
        assemble_instruction(
            &mut ops,
            inst,
            label,
            AssemblyKind::Oneoff,
            bus,
            options.trace_pc,
        );
    });

    generate_offset_table(&mut ops, labels.as_slice());

    let table_offset = ops
        .labels()
        .resolve_global("jump_table")
        .expect("jump_table should exist");

    ops.commit()
        .expect("No assembly errors should have occurred");

    let buf = ops.finalize().expect("No executor instances created");

    Ok((buf, table_offset))
}

fn generate_boilerplate(ops: &mut Assembler) -> AssemblyOffset {
    // Entry has type: fn (cpu_state: *mut CpuState, target_pc: u64, parameter: *mut c_void)
    let offset = ops.offset();
    dynasm!(ops
        ; -> block_start:
        ; push rbp
        ; mov [rsp - 0x08], r12
        ; mov [rsp - 0x10], r13
        ; mov [rsp - 0x18], r14
        ; mov [rsp - 0x20], r15
        ; mov [rsp - 0x28], rbx
        ; sub rsp, 0x60
        ; mov rbp, rsi
        ;; setup_cycle_registers(ops)
        ;; unpack_cpu_state(ops)
        ;; setup_limit_address(ops)
        ; mov [rsp + 0x08], rdi
        ; jmp -> jump
        ; -> exit:
        ; mov rdi, [rsp + 0x08]
        ;; repack_cpu_state(ops)
        ; add rsp, 0x60
        ; mov r12, [rsp - 0x08]
        ; mov r13, [rsp - 0x10]
        ; mov r14, [rsp - 0x18]
        ; mov r15, [rsp - 0x20]
        ; mov rbx, [rsp - 0x28]
        ; pop rbp
        ; ret
    );
    offset
}

fn setup_cycle_registers(ops: &mut Assembler) {
    dynasm!(ops
        ; mov r14, [rdx + 0x00]
        ; mov r8, [rdx + 0x08]
        ; mov [rsp + 0x20], r8
        ; mov r8, [rdx + 0x10]
        ; mov [rsp + 0x28], r8
    );
}

fn generate_overrun(ops: &mut Assembler) {
    dynasm!(ops
        ; jmp -> exit
    )
}

type Generator = fn(&mut Assembler, &Instruction, &ExternalBus) -> EpilogueDescription;

#[derive(Debug, Clone, Copy)]
enum JumpDescription {
    Static(u16),
    Relative(i8),
    Dynamic, // Target should be put in di
}

#[derive(Debug, Clone, Copy)]
enum EpilogueDescription {
    Default,
    Jump {
        target: JumpDescription,
        skip_label: Option<DynamicLabel>,
    },
}

impl Default for EpilogueDescription {
    fn default() -> Self {
        EpilogueDescription::Default
    }
}

enum AssemblyKind<'a> {
    Static {
        base_addr: u16,
        pc: u16,
        labels: &'a [DynamicLabel],
    },
    Oneoff,
}

fn assemble_instruction<'a>(
    ops: &mut Assembler,
    inst: &Instruction,
    label: &DynamicLabel,
    kind: AssemblyKind<'a>,
    bus: &ExternalBus,
    trace_pc: bool,
) -> AssemblyOffset {
    let offset = ops.offset();
    dynasm!(ops
        ; => *label
    );

    match kind {
        AssemblyKind::Static {
            base_addr: _,
            pc: _,
            labels: _,
        } => (),
        AssemblyKind::Oneoff => {
            dynasm!(ops
                ; pop r8
                ; mov [rsp + 0x18], r8
            );
        }
    }

    if trace_pc {
        dynasm!(ops
            ;; push_state(ops)
            ; mov rdi, [rsp + 0x08]
            ;; repack_cpu_state(ops)
            ; mov rsi, inst.encoding[0] as _
            ; mov rdx, [r14]
            ; mov rax, QWORD log_state as _
            ; call rax
            ;; pop_state(ops)
        );
    }

    let generator: Generator = {
        use Command::*;
        match inst.cmd {
            LdHalf { src: _, dst: _ } => ldhalf::generate,
            LdAddrInc { inc: _, load: _ } => ldaddrinc::generate,
            LdFullImm { dst: _, val: _ } => ldfullimm::generate,
            StoreSp { addr: _ } => storesp::generate,
            Push(_) => push::generate,
            Pop(_) => pop::generate,
            AluHalf { cmd: _, op: _ } => aluhalf::generate,
            Daa => daa::generate,
            Cpl => cpl::generate,
            AddHl(_) => addhl::generate,
            IncDecHalf { loc: _, inc: _ } => incdechalf::generate,
            IncDecFull { reg: _, inc: _ } => incdecfull::generate,
            AddSp(_) => addsp::generate,
            HlSpOffset(_) => hlspoffset::generate,
            LdSpHl => ldsphl::generate,
            BitHalf { cmd: _, op: _ } => bithalf::generate,
            Control(_) => control::generate,
            Jump {
                target: _,
                condition: _,
            } => jump::generate,
            Call {
                target: _,
                condition: _,
            } => call::generate,
            Ret {
                condition: _,
                intenable: _,
            } => ret::generate,
            Rst(_) => rst::generate,
            _ => generate_invalid,
        }
    };

    let epilogue_desc = generator(ops, inst, bus);

    match kind {
        AssemblyKind::Static {
            base_addr,
            pc,
            labels,
        } => generate_epilogue(ops, &epilogue_desc, inst, labels, pc, base_addr),
        AssemblyKind::Oneoff => generate_oneoff_epilogue(ops, &epilogue_desc, inst),
    }

    offset
}

fn assemble_incomplete(
    ops: &mut Assembler,
    bytes: &[u8],
    label: &DynamicLabel,
    pc: u16,
    bus: &ExternalBus,
    oneoffs: &OneoffTable,
) -> AssemblyOffset {
    let offset = ops.offset();
    dynasm!(ops
        ; => *label
    );

    let req = decoder::bytes_required(bytes[0]);
    dynasm!(ops
        ; mov WORD [rsp + 0x00], WORD 0
    );

    if req >= 2 {
        if bytes.len() >= 2 {
            dynasm!(ops
                ; mov BYTE [rsp + 0x00], BYTE bytes[1] as _
            );
        } else {
            dynasm!(ops
                ; mov di, WORD pc.wrapping_add(1) as _
                ;; call_read(ops, bus)
                ; mov [rsp + 0x00], ah
            );
        }
    }

    if req >= 3 {
        if bytes.len() >= 3 {
            dynasm!(ops
                ; mov BYTE [rsp + 0x01], BYTE bytes[2] as _
            );
        } else {
            dynasm!(ops
                ; mov di, WORD pc.wrapping_add(2) as _
                ;; call_read(ops, bus)
                ; mov [rsp + 0x01], ah
            );
        }
    }

    // The index is now in [rsp]

    let table = oneoffs.get_table(bytes[0]);

    dynasm!(ops
        ; mov rdi, 0
        ; mov di, [rsp + 0x00]
        ; shl rdi, 3
        ; mov r8, QWORD table.table() as _
        ; mov r9, QWORD table.base() as _
        ; add rdi, r8
        ; mov rsi, [r8]
        ; add rsi, r9
        ; call rsi
        ;; check_cycle_limit(ops)
        ; jmp -> jump
    );

    offset
}

fn generate_epilogue(
    ops: &mut Assembler,
    desc: &EpilogueDescription,
    inst: &Instruction,
    labels: &[DynamicLabel],
    pc: u16,
    base_addr: u16,
) {
    match desc {
        EpilogueDescription::Default => generate_static_jump_epilogue(
            ops,
            inst.cycles,
            pc,
            pc.wrapping_add(inst.size()),
            base_addr,
            labels,
        ),
        EpilogueDescription::Jump { target, skip_label } => {
            match target {
                JumpDescription::Static(target_pc) => generate_static_jump_epilogue(
                    ops,
                    inst.cycles,
                    pc,
                    *target_pc,
                    base_addr,
                    labels,
                ),
                JumpDescription::Relative(offset) => generate_static_jump_epilogue(
                    ops,
                    inst.cycles,
                    pc,
                    pc.wrapping_add(inst.size()).wrapping_add(*offset as u16),
                    base_addr,
                    labels,
                ),
                JumpDescription::Dynamic => generate_dynamic_jump_epilogue(ops, inst.cycles),
            }
            if let Some(label) = skip_label {
                dynasm!(ops
                    ; => *label
                );
                generate_static_jump_epilogue(
                    ops,
                    inst.alt_cycles.unwrap(),
                    pc,
                    pc.wrapping_add(inst.size()),
                    base_addr,
                    labels,
                );
            }
        }
    }
}

fn generate_oneoff_epilogue(ops: &mut Assembler, desc: &EpilogueDescription, inst: &Instruction) {
    let epilogue = |ops: &mut Assembler| {
        dynasm!(ops
            ; add QWORD [r14], DWORD inst.cycles as _
            ; mov r8, [rsp + 0x18]
            ; push r8
            ; ret
        );
    };
    match desc {
        EpilogueDescription::Default => {
            dynasm!(ops
                ; add r13w, WORD inst.size() as _
                ;; epilogue(ops)
            );
        }
        EpilogueDescription::Jump { target, skip_label } => {
            match target {
                JumpDescription::Static(target_pc) => {
                    dynasm!(ops
                        ; add r13w, WORD *target_pc as _
                        ;; epilogue(ops)
                    );
                }
                JumpDescription::Relative(offset) => {
                    let offset = inst.size().wrapping_add(*offset as u16);
                    dynasm!(ops
                        ; add r13w, WORD offset as _
                        ;; epilogue(ops)
                    );
                }
                JumpDescription::Dynamic => {
                    dynasm!(ops
                        ; mov r13w, di
                        ;; epilogue(ops)
                    );
                }
            }
            if let Some(label) = skip_label {
                dynasm!(ops
                    ; => *label
                    ; add r13w, WORD inst.size() as _
                    ;; epilogue(ops)
                );
            }
        }
    }
}

fn generate_static_jump_epilogue(
    ops: &mut Assembler,
    cycles: u8,
    pc: u16,
    target_pc: u16,
    base_addr: u16,
    labels: &[DynamicLabel],
) {
    dynasm!(ops
        ; mov r13w, WORD target_pc as _
        ; add QWORD [r14], DWORD cycles as _
        ;; check_cycle_limit(ops)
    );
    if target_pc != pc.wrapping_add(1) {
        util::direct_jump(ops, target_pc, labels, base_addr);
    }
}

fn generate_dynamic_jump_epilogue(ops: &mut Assembler, cycles: u8) {
    dynasm!(ops
        ; mov r13w, di
        ; add QWORD [r14], DWORD cycles as _
        ;; check_cycle_limit(ops)
        ; jmp -> jump
    );
}

fn generate_dynamic_jump_table(ops: &mut Assembler, base_addr: u16, labels: &[DynamicLabel]) {
    dynasm!(ops
        ; -> jump:
        ; mov di, r13w
        ; sub di, WORD base_addr as _
        ; cmp di, WORD labels.len() as _
        ; jae -> exit
        ; and rdi, DWORD 0xffff as _
        ; shl rdi, 3
        ; lea r8, [-> jump_table]
        ; add r8, rdi
        ; mov rsi, [r8]
        ; lea r9, [-> block_start]
        ; add rsi, r9
        ; jmp rsi
    );

    generate_offset_table(ops, labels);
}

fn generate_offset_table(ops: &mut Assembler, labels: &[DynamicLabel]) {
    dynasm!(ops
        ; -> jump_table:
    );

    let offsets: Vec<AssemblyOffset> = {
        let registry = ops.labels();
        labels
            .iter()
            .map(|x| registry.resolve_dynamic(*x).unwrap())
            .collect()
    };

    for offset in offsets.iter() {
        dynasm!(ops
            ; .qword offset.0 as _
        );
    }
}

fn check_cycle_limit(ops: &mut Assembler) {
    dynasm!(ops
        ; mov r8, [r15]
        ; cmp [r14], r8
        ; jge -> exit
    );
}

fn generate_invalid(
    ops: &mut Assembler,
    inst: &Instruction,
    _bus: &ExternalBus,
) -> EpilogueDescription {
    dynasm!(ops
        ;; push_state(ops)
        ; mov rax, QWORD log_invalid as _
        ; mov di, r13w
        ; mov rsi, QWORD inst.encoding[0] as _
        ; call rax
        ;; pop_state(ops)
    );

    Default::default()
}

extern "sysv64" fn log_invalid(pc: u16, opcode: u8) {
    warn!(
        "Executing invalid instruction at {:#06x?}, opcode {:#04x?}",
        pc, opcode
    );
}

extern "sysv64" fn log_state(state: *const c_void, opcode: u8, cycle: u64) {
    let state: &CpuState = unsafe { &*(state as *const CpuState) };
    let pc = state.pc;
    trace!(
        "Executing instruction {:#04x?} at {:#06x?}, state: {:?}, cycle: {:?}",
        opcode,
        pc,
        state,
        cycle
    );
}

extern "sysv64" fn log_registers(regs: *const u64) {
    let names = ["rax", "rbx", "rcx", "rdx", "rdi", "rsi", "r12", "r13"];
    let regs = unsafe { std::slice::from_raw_parts(regs, names.len()) };

    log::trace!("Print registers");
    for (name, reg) in names.iter().zip(regs.iter()) {
        log::trace!("Reg {}: {:016x}", name, reg);
    }
}
