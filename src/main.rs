#![allow(clippy::match_bool, clippy::fn_to_numeric_cast)]
#![feature(proc_macro_hygiene)]
extern crate dynasm;

use std::mem;

use dynasm::dynasm;
use dynasmrt::{x64, DynasmApi, DynasmLabelApi};

mod compiler;

fn main() {
    let mut ops = x64::Assembler::new().unwrap();

    let msg = "goodbye world\n";

    dynasm!(ops
        ; .arch x64
        ; msg_label:
        ; .bytes msg.as_bytes()
    );

    let goodbye_addr = ops.offset();
    dynasm!(ops
        ; mov rax, QWORD 0
        ; mov al, BYTE 0xff as _
        ; mov ah, BYTE 1
        ; add ah, al
        ; pushf
        ; pop rdi
        ; mov rax, QWORD printnum as _
        ; sub rsp, BYTE 0x8
        ; call rax
        ; add rsp, BYTE 0x8
        ; ret
    );

    let buf = ops.finalize().unwrap();
    let goodbye_fun: extern "sysv64" fn() -> bool =
        unsafe { mem::transmute(buf.ptr(goodbye_addr)) };

    assert!(goodbye_fun());
}

extern "sysv64" fn printnum(a: u64) -> bool {
    println!("Number: {:08x}", a);
    true
}
