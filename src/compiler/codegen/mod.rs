use dynasm::dynasm;
use dynasmrt::{x64, AssemblyOffset, DynamicLabel, DynasmApi, DynasmLabelApi, ExecutableBuffer};
use log::*;

use super::external_bus::TypeErased as ExternalBus;
use super::instruction::*;
use super::CompileError;

pub fn codegen(
    base_addr: u16,
    insts: &[Instruction],
    bus: &ExternalBus,
) -> Result<(ExecutableBuffer, AssemblyOffset, Vec<AssemblyOffset>), CompileError> {
    let mut ops = x64::Assembler::new()?;

    let entry = generate_boilerplate(&mut ops);

    let len = insts.len() as u16;
    let labels: Vec<DynamicLabel> = (base_addr..base_addr + len)
        .map(|_| ops.new_dynamic_label())
        .collect();

    let offset = assemble_instruction(
        &mut ops,
        &insts[0],
        labels.as_slice(),
        base_addr,
        base_addr,
        bus,
    );

    generate_overrun(&mut ops);

    let offsets = vec![offset];

    ops.commit()
        .expect("No assembly errors should have occurred");

    let buf = ops.finalize().expect("No executor instances created");

    Ok((buf, entry, offsets))
}

fn generate_boilerplate(ops: &mut x64::Assembler) -> AssemblyOffset {
    // Entry has type: fn (cpu_state: *mut CpuState, target_pc: u64, parameter: *mut c_void)
    let offset = ops.offset();
    dynasm!(ops
        ; push rbp
        ; mov [rsp - 0x08], r12
        ; mov [rsp - 0x10], r13
        ; mov [rsp - 0x18], r14
        ; mov [rsp - 0x20], r15
        ; sub rsp, 0x30
        ; mov rbp, rdx
        ; mov r14, [rdi + 0x00] // cycles
        ; mov r12w, [rdi + 0x08] // sp
        ; mov r13w, [rdi + 0x0a] // pc
        ; mov ax, [rdi + 0x0c] // af
        ; mov bx, [rdi + 0x0e] // bc
        ; mov cx, [rdi + 0x10] // de
        ; mov dx, [rdi + 0x12] // hl
        ; mov [rsp + 0x00], rdi
        ; mov [rsp + 0x08], rdx
        ; jmp rsi
        ; -> exit:
        ; mov rdi, [rsp + 0x00]
        ; mov [rdi + 0x00], r14 // cycles
        ; mov [rdi + 0x08], r12w // sp
        ; mov [rdi + 0x0a], r13w // pc
        ; mov [rdi + 0x0c], ax // af
        ; mov [rdi + 0x0e], bx // bc
        ; mov [rdi + 0x10], cx // de
        ; mov [rdi + 0x12], dx // hl
        ; add rsp, 0x30
        ; mov r12, [rsp - 0x08]
        ; mov r13, [rsp - 0x10]
        ; mov r14, [rsp - 0x18]
        ; mov r15, [rsp - 0x20]
        ; pop rbp
        ; ret
    );
    offset
}

fn generate_overrun(ops: &mut x64::Assembler) {
    dynasm!(ops
        ; jmp ->exit
    )
}

fn label(pc: u16) -> String {
    format!("inst{:04x}", pc)
}

type GenerateEpilogue = bool;
type Generator = fn(
    &mut x64::Assembler,
    &Instruction,
    labels: &[DynamicLabel],
    pc: u16,
    base_addr: u16,
    bus: &ExternalBus,
) -> GenerateEpilogue;

fn assemble_instruction(
    ops: &mut x64::Assembler,
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

    let generator = match inst.cmd {
        _ => generate_invalid,
    };

    let generate_epilogue = generator(ops, inst, labels, pc, base_addr, bus);

    if generate_epilogue {
        dynasm!(ops
            ; add r14, DWORD inst.cycles as _
        );
    }

    offset
}

fn push_state(ops: &mut x64::Assembler) {
    // TODO: add r11 when we start using that
    dynasm!(ops
        ; push rax
        ; push rcx
        ; push rdx
        ; sub rsp, 8
    );
}

fn pop_state(ops: &mut x64::Assembler) {
    // TODO: add r11 when we start using that
    dynasm!(ops
        ; add rsp, 8
        ; pop rax
        ; pop rcx
        ; pop rdx
    );
}

fn generate_invalid(
    ops: &mut x64::Assembler,
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
