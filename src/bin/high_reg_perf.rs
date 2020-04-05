#![allow(clippy::fn_to_numeric_cast)]
#![feature(proc_macro_hygiene)]

/// Designed to evaluate the performance of different approaches to handling the high bit
/// registers.
extern crate dynasm;

use std::mem;
use std::time;

use dynasm::dynasm;
use dynasmrt::{x64, AssemblyOffset, DynasmApi, DynasmLabelApi};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ops = x64::Assembler::new()?;

    let xchg_offset = create_bench(&mut ops, bench_xchg);
    let stack_offset = create_bench(&mut ops, bench_stack);
    let stc_offset = create_bench(&mut ops, bench_stc);
    let add_offset = create_bench(&mut ops, bench_add);
    let sahf_offset = create_bench(&mut ops, bench_sahf);

    let buf = ops.finalize().unwrap();
    let xchg: extern "sysv64" fn(u64) = unsafe { mem::transmute(buf.ptr(xchg_offset)) };
    let stack: extern "sysv64" fn(u64) = unsafe { mem::transmute(buf.ptr(stack_offset)) };
    let stc: extern "sysv64" fn(u64) = unsafe { mem::transmute(buf.ptr(stc_offset)) };
    let add: extern "sysv64" fn(u64) = unsafe { mem::transmute(buf.ptr(add_offset)) };
    let sahf: extern "sysv64" fn(u64) = unsafe { mem::transmute(buf.ptr(sahf_offset)) };

    let iter = 100_000_000;
    println!("Xchg: {:#?}", time_execution(|| xchg(iter)));
    println!("Stack: {:#?}", time_execution(|| stack(iter)));
    println!("Stc: {:#?}", time_execution(|| stc(iter)));
    println!("Add: {:#?}", time_execution(|| add(iter)));
    println!("Sahf: {:#?}", time_execution(|| sahf(iter)));

    Ok(())
}

fn create_bench(ops: &mut x64::Assembler, f: impl FnOnce(&mut x64::Assembler)) -> AssemblyOffset {
    let offset = ops.offset();
    dynasm!(ops
        ; sub rsp, 8
        ; mov BYTE [rsp + 2], 1
        ; start:
        ;; f(ops)
        ; sub rdi, 1
        ; jnz <start
        ; add rsp, 8
        ; ret
    );
    offset
}

fn time_execution(f: impl FnOnce()) -> time::Duration {
    let start = time::Instant::now();
    f();
    time::Instant::now() - start
}

fn bench_xchg(ops: &mut x64::Assembler) {
    dynasm!(ops
        ; xchg ch, cl
        ; mov sil, cl
        ; xchg ch, cl
        ; xchg dh, dl
        ; mov dl, sil
        ; xchg dh, dl
    )
}

fn bench_stack(ops: &mut x64::Assembler) {
    dynasm!(ops
        ; mov [rsp], ah
        ; mov ah, ch
        ; mov dh, ah
        ; mov ah, [rsp]
    )
}

fn bench_stc(ops: &mut x64::Assembler) {
    dynasm!(ops
        ; test [rsp + 2], 1
        ; jz >l1
        ; stc
        ; jmp >l2
        ; l1:
        ; clc
        ; l2:
        ; adc al, ah
    )
}

fn bench_add(ops: &mut x64::Assembler) {
    dynasm!(ops
        ; mov [rsp], ah
        ; mov ah, [rsp + 2]
        ; and ah, 1
        ; add ah, BYTE 255 as _
        ; mov ah, [rsp]
        ; adc al, ah
    )
}

fn bench_sahf(ops: &mut x64::Assembler) {
    dynasm!(ops
        ; mov [rsp], ah
        ; mov ah, [rsp + 2]
        ; sahf
        ; mov ah, [rsp]
        ; adc al, ah
    )
}
