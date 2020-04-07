use dynasm::dynasm;
use dynasmrt::x64::Assembler;
use dynasmrt::{AssemblyOffset, DynamicLabel, DynasmApi, DynasmLabelApi, ExecutableBuffer};
use log::*;

use super::external_bus::TypeErased as ExternalBus;
use super::instruction::{self, *};
use super::CompileError;

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
mod storesp;

use util::{pop_state, push_state};

pub fn codegen(
    base_addr: u16,
    insts: &[Instruction],
    bus: &ExternalBus,
) -> Result<(ExecutableBuffer, AssemblyOffset, Vec<AssemblyOffset>), CompileError> {
    let mut ops = Assembler::new()?;

    let entry = generate_boilerplate(&mut ops);

    let len = insts.len() as u16;
    let labels: Vec<DynamicLabel> = (base_addr..base_addr + len)
        .map(|_| ops.new_dynamic_label())
        .collect();

    let offsets = (base_addr..base_addr + len)
        .map(|pc| {
            assemble_instruction(
                &mut ops,
                &insts[pc as usize],
                labels.as_slice(),
                pc,
                base_addr,
                bus,
            )
        })
        .collect();

    generate_overrun(&mut ops);

    ops.commit()
        .expect("No assembly errors should have occurred");

    let buf = ops.finalize().expect("No executor instances created");

    Ok((buf, entry, offsets))
}

fn generate_boilerplate(ops: &mut Assembler) -> AssemblyOffset {
    // Entry has type: fn (cpu_state: *mut CpuState, target_pc: u64, parameter: *mut c_void)
    let offset = ops.offset();
    dynasm!(ops
        ; push rbp
        ; mov [rsp - 0x08], r12
        ; mov [rsp - 0x10], r13
        ; mov [rsp - 0x18], r14
        ; mov [rsp - 0x20], r15
        ; sub rsp, 0x40
        ; mov rbp, rdx
        ; mov r14, [rdi + 0x00] // cycles
        ; mov r12w, [rdi + 0x08] // sp
        ; mov r13w, [rdi + 0x0a] // pc
        ; mov ax, [rdi + 0x0c] // af
        ; mov [rsp + 0x02], ah // f
        ; mov bx, [rdi + 0x0e] // bc
        ; mov cx, [rdi + 0x10] // de
        ; mov dx, [rdi + 0x12] // hl
        ; mov [rsp + 0x08], rdi
        ; mov [rsp + 0x10], rdx
        ; jmp rsi
        ; -> exit:
        ; mov rdi, [rsp + 0x08]
        ; mov [rdi + 0x00], r14 // cycles
        ; mov [rdi + 0x08], r12w // sp
        ; mov [rdi + 0x0a], r13w // pc
        ; mov ah, [rsp + 0x02] // f
        ; mov [rdi + 0x0c], ax // af
        ; mov [rdi + 0x0e], bx // bc
        ; mov [rdi + 0x10], cx // de
        ; mov [rdi + 0x12], dx // hl
        ; add rsp, 0x40
        ; mov r12, [rsp - 0x08]
        ; mov r13, [rsp - 0x10]
        ; mov r14, [rsp - 0x18]
        ; mov r15, [rsp - 0x20]
        ; pop rbp
        ; ret
    );
    offset
}

fn generate_overrun(ops: &mut Assembler) {
    dynasm!(ops
        ; jmp ->exit
    )
}

fn label(pc: u16) -> String {
    format!("inst{:04x}", pc)
}

type Generator = fn(
    &mut Assembler,
    &Instruction,
    labels: &[DynamicLabel],
    pc: u16,
    base_addr: u16,
    bus: &ExternalBus,
) -> EpilogueDescription;

#[derive(Debug, Clone, Copy)]
enum JumpDescription {
    Static(u16),
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

fn assemble_instruction(
    ops: &mut Assembler,
    inst: &Instruction,
    labels: &[DynamicLabel],
    pc: u16,
    base_addr: u16,
    bus: &ExternalBus,
) -> AssemblyOffset {
    let offset = ops.offset();
    dynasm!(ops
        ; => labels[(pc-base_addr) as usize]
    );

    let generator = {
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
            _ => generate_invalid,
        }
    };

    let epilogue_desc = generator(ops, inst, pc, bus);

    generate_epilogue(ops, &epilogue_desc, inst, labels, pc, base_addr);

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
                JumpDescription::Dynamic => {
                    generate_dynamic_jump_epilogue(ops, inst.cycles, pc, base_addr, labels)
                }
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
        ; add r14, DWORD cycles as _
    );
    if target_pc != pc.wrapping_add(1) {
        util::direct_jump(ops, target_pc, labels, base_addr);
    }
}

fn generate_dynamic_jump_epilogue(
    _ops: &mut Assembler,
    _cycles: u8,
    _pc: u16,
    _base_addr: u16,
    _labels: &[DynamicLabel],
) {
    unimplemented!()
}

fn generate_invalid(
    ops: &mut Assembler,
    inst: &Instruction,
    pc: u16,
    _bus: &ExternalBus,
) -> EpilogueDescription {
    dynasm!(ops
        ;; push_state(ops)
        ; mov rax, QWORD log_invalid as _
        ; mov rdi, QWORD pc as _
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
