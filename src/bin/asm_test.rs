#![feature(proc_macro_hygiene)]
extern crate dynasm;
use std::mem;

use dynasm::dynasm;
use dynasmrt::{x64, DynasmApi, DynasmLabelApi};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ops = x64::Assembler::new().unwrap();

    dynasm!(ops
        ; .arch x64
        ; print_flags:
        ; lahf
        ; mov rdi, 0
        ; mov di, ax
        ; shr rdi, 8
        ; mov rax, QWORD print as _
        ; call rax
        ; ret
        ; print_num:
        ; mov rax, QWORD print as _
        ; call rax
        ; ret
    );

    let fn_addr = ops.offset();
    dynasm!(ops
        ; mov rbx, QWORD 0
        ; mov bl, BYTE 246u8 as i8
        ; mov bh, BYTE 9
        ; add bh, bl
        ; mov r12, rbx
        ; call <print_flags
        ; mov rdi, r12
        ; call <print_num
        ; mov bx, WORD 0xa0c
        ; mov cx, WORD 0x506
        ; add bx, cx
        ; mov r12, rbx
        ; call <print_flags
        ; mov rdi, r12
        ; call <print_num
        ; ret
    );

    let buf = ops.finalize().unwrap();
    let fun: extern "sysv64" fn() -> bool = unsafe { mem::transmute(buf.ptr(fn_addr)) };

    if fun() {
        Ok(())
    } else {
        Err("goodbye fun failed".into())
    }
}

extern "sysv64" fn print(s: u64) {
    println!("{:#018x?}", s);
}
