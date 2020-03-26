#![feature(proc_macro_hygiene)]
extern crate dynasm;
extern crate dynasmrt;

use std::{io, mem, slice};

use std::io::Write;

use dynasm::dynasm;
use dynasmrt::{x64, DynasmApi, DynasmLabelApi};

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
        ; lea rdi, [<msg_label]
        ; mov rsi, BYTE msg.len() as _
        ; mov rax, QWORD print as _
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

pub extern "sysv64" fn print(s: *const u8, len: u64) -> bool {
    print2(s, len)
}

pub extern "win64" fn print2(s: *const u8, len: u64) -> bool {
    io::stdout()
        .write_all(unsafe { slice::from_raw_parts(s, len as usize) })
        .is_ok()
}
