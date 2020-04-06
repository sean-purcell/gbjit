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
mod cpl;
mod daa;
mod hlspoffset;
mod incdecfull;
mod incdechalf;
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

type GenerateEpilogue = bool;
type Generator = fn(
    &mut Assembler,
    &Instruction,
    labels: &[DynamicLabel],
    pc: u16,
    base_addr: u16,
    bus: &ExternalBus,
) -> GenerateEpilogue;

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
            _ => generate_invalid,
        }
    };

    let generate_epilogue = generator(ops, inst, labels, pc, base_addr, bus);

    if generate_epilogue {
        let target_pc = pc + inst.size();
        dynasm!(ops
            ; mov r13w, WORD target_pc as _
            ; add r14, DWORD inst.cycles as _
        );
        if inst.size() != 1 {
            let target_idx = pc + inst.size() - base_addr;
            if target_idx >= labels.len() as u16 {
                dynasm!(ops
                    ; jmp ->exit
                );
            } else {
                dynasm!(ops
                    ; jmp =>labels[target_idx as usize]
                );
            }
        }
    }

    offset
}

fn generate_invalid(
    ops: &mut Assembler,
    inst: &Instruction,
    _labels: &[DynamicLabel],
    pc: u16,
    _base_addr: u16,
    _bus: &ExternalBus,
) -> GenerateEpilogue {
    dynasm!(ops
        ;; push_state(ops)
        ; mov rax, QWORD log_invalid as _
        ; mov rdi, QWORD pc as _
        ; mov rsi, QWORD inst.encoding[0] as _
        ; call rax
        ;; pop_state(ops)
    );

    true
}

extern "sysv64" fn log_invalid(pc: u16, opcode: u8) {
    warn!(
        "Executing invalid instruction at {:#06x?}, opcode {:#04x?}",
        pc, opcode
    );
}
