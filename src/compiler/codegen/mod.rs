use dynasm::dynasm;
use dynasmrt::{x64, AssemblyOffset, DynasmApi, DynasmLabelApi};

use super::instruction::*;

use super::CodeBlock;

pub fn codegen(
    base_addr: u16,
    insts: &[Instruction],
) -> Result<CodeBlock, Box<dyn std::error::Error>> {
    let mut ops = x64::Assembler::new()?;

    let entry = generate_boilerplate(&mut ops);

    ops.commit()?;

    let buf = ops.finalize().expect("No executor instances created");

    Ok(CodeBlock::new(
        base_addr,
        buf,
        entry,
        Vec::new(),
        insts.to_vec(),
    ))
}

fn generate_boilerplate(ops: &mut x64::Assembler) -> AssemblyOffset {
    let offset = ops.offset();
    dynasm!(ops
        ; push rbp
        ; mov [rsp - 0x08], r12
        ; mov [rsp - 0x10], r13
        ; mov [rsp - 0x18], r14
        ; mov [rsp - 0x20], r15
        ; sub rsp, 0x20
        ; -> exit:
        ; add rsp, 0x20
        ; mov r12, [rsp - 0x08]
        ; mov r13, [rsp - 0x10]
        ; mov r14, [rsp - 0x18]
        ; mov r15, [rsp - 0x20]
        ; pop rbp
        ; ret
    );
    offset
}
