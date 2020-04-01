use dynasm::dynasm;
use dynasmrt::{x64, AssemblyOffset, DynasmApi, DynasmLabelApi, ExecutableBuffer};

use super::external_bus::TypeErased as ExternalBus;
use super::instruction::*;
use super::CompileError;

pub fn codegen(
    base_addr: u16,
    insts: &[Instruction],
    _bus: &ExternalBus,
) -> Result<(ExecutableBuffer, AssemblyOffset, Vec<AssemblyOffset>), CompileError> {
    let mut ops = x64::Assembler::new()?;

    let entry = generate_boilerplate(&mut ops);

    let offset = assemble_instruction(
        &mut ops,
        &insts[0],
        base_addr,
        base_addr,
        insts.len() as u16,
    );
    let offsets = vec![offset];

    ops.commit()
        .expect("No assembly errors should have occurred");

    let buf = ops.finalize().expect("No executor instances created");

    Ok((buf, entry, offsets))
}

fn generate_boilerplate(ops: &mut x64::Assembler) -> AssemblyOffset {
    // Entry has type: fn (cpu_state: *mut CpuState, target_pc: u64)
    let offset = ops.offset();
    dynasm!(ops
        ; push rbp
        ; mov [rsp - 0x08], r12
        ; mov [rsp - 0x10], r13
        ; mov [rsp - 0x18], r14
        ; mov [rsp - 0x20], r15
        ; sub rsp, 0x30
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

type Generator = fn(&mut x64::Assembler, &Instruction, pc: u16, base_addr: u16, len: u16);

fn assemble_instruction(
    ops: &mut x64::Assembler,
    _inst: &Instruction,
    _pc: u16,
    _base_addr: u16,
    _len: u16,
) -> AssemblyOffset {
    let offset = ops.offset();
    dynasm!(ops
        ; add bl, 1
        ; lahf
        ; jmp ->exit
    );
    offset
}
