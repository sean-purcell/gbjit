pub use dynasm::dynasm;
pub use dynasmrt::x64::Assembler;
pub use dynasmrt::{DynamicLabel, DynasmApi, DynasmLabelApi};

pub(super) use super::instruction::*;
pub(super) use super::Command::*;
pub(super) use super::{EpilogueDescription, ExternalBus, JumpDescription};

macro_rules! parse_cmd {
    ($i:expr, $p:pat => $e:expr) => {
        if let $p = ($i).cmd {
            $e
        } else {
            panic!("Wrong pattern provided")
        }
    };
}

pub fn unpack_cpu_state(ops: &mut Assembler) {
    dynasm!(ops
        ; mov r12w, [rdi + 0x00] // sp
        ; mov r13w, [rdi + 0x02] // pc
        ; mov ax, [rdi + 0x04] // af
        ; mov [rsp + 0x02], ah // f
        ; mov bx, [rdi + 0x06] // bc
        ; mov cx, [rdi + 0x08] // de
        ; mov dx, [rdi + 0x0a] // hl
        ; mov r11b, [rdi + 0x0c] // intenable
    );
}

pub fn repack_cpu_state(ops: &mut Assembler) {
    dynasm!(ops
        ; mov [rdi + 0x00], r12w // sp
        ; mov [rdi + 0x02], r13w // pc
        ; mov ah, [rsp + 0x02] // f
        ; mov [rdi + 0x04], ax // af
        ; mov [rdi + 0x06], bx // bc
        ; mov [rdi + 0x08], cx // de
        ; mov [rdi + 0x0a], dx // hl
        ; mov [rdi + 0x0c], r11b // intenable
    );
}

pub fn setup_limit_address(ops: &mut Assembler) {
    dynasm!(ops
        ; test r11b, r11b
        ; cmovz r15, [rsp + 0x20]
        ; cmovnz r15, [rsp + 0x28]
    );
}

pub fn push_state(ops: &mut Assembler) {
    // TODO: add r11 when we start using that
    dynasm!(ops
        ; mov [rsp + 0x03], al
        ; mov [rsp + 0x04], cx
        ; mov [rsp + 0x06], dx
    );
}

pub fn pop_state(ops: &mut Assembler) {
    // TODO: add r11 when we start using that
    dynasm!(ops
        ; mov al, [rsp + 0x03]
        ; mov cx, [rsp + 0x04]
        ; mov dx, [rsp + 0x06]
    );
}

pub fn call_read(ops: &mut Assembler, bus: &ExternalBus) {
    dynasm!(ops
        ;; push_state(ops)
        ; mov rsi, rbp
        ; mov rax, QWORD bus.read as _
        ; call rax
        ; mov ah, al
        ;; pop_state(ops)
    );
}

pub fn call_write(ops: &mut Assembler, bus: &ExternalBus) {
    dynasm!(ops
        ;; push_state(ops)
        ; mov rdx, rbp
        ; mov rax, QWORD bus.write as _
        ; call rax
        ;; pop_state(ops)
    );
}

pub fn call_interrupts(ops: &mut Assembler, bus: &ExternalBus) {
    dynasm!(ops
        ;; push_state(ops)
        ; mov rsi, rbp
        ; mov rax, QWORD bus.write as _
        ; call rax
        ;; pop_state(ops)
    );
}

pub fn load_halfreg(ops: &mut Assembler, r: HalfReg) {
    macro_rules! ld {
        ($ops:expr, $r:tt) => {
            dynasm!($ops
                ; mov ah, $r
            )
        };
    }

    use HalfReg::*;
    match r {
        A => ld!(ops, al),
        B => ld!(ops, bh),
        C => ld!(ops, bl),
        D => ld!(ops, ch),
        E => ld!(ops, cl),
        H => ld!(ops, dh),
        L => ld!(ops, dl),
    }
}

pub fn store_halfreg(ops: &mut Assembler, r: HalfReg) {
    macro_rules! st {
        ($ops:expr, $r:tt) => {
            dynasm!($ops
                ; mov $r, ah
            )
        };
    }

    use HalfReg::*;
    match r {
        A => st!(ops, al),
        B => st!(ops, bh),
        C => st!(ops, bl),
        D => st!(ops, ch),
        E => st!(ops, cl),
        H => st!(ops, dh),
        L => st!(ops, dl),
    }
}

pub fn load_location(ops: &mut Assembler, bus: &ExternalBus, loc: Location) {
    use Location::*;
    match loc {
        Reg(r) => load_halfreg(ops, r),
        Mem => {
            dynasm!(ops
                ; mov di, dx
                ;; call_read(ops, bus)
            );
        }
    }
}

pub fn store_location(ops: &mut Assembler, bus: &ExternalBus, loc: Location) {
    use Location::*;
    match loc {
        Reg(r) => store_halfreg(ops, r),
        Mem => {
            dynasm!(ops
                ; mov di, dx
                ; mov [rsp], ah
                ; mov sil, [rsp]
                ;; call_write(ops, bus)
            );
        }
    }
}

pub fn load_reg(ops: &mut Assembler, r: Reg) {
    macro_rules! ld {
        ($ops:expr, $r:tt) => {
            dynasm!($ops
                ; mov di, $r
            )
        };
    }

    use Reg::*;
    match r {
        AF => materialize_af(ops),
        BC => ld!(ops, bx),
        DE => ld!(ops, cx),
        HL => ld!(ops, dx),
        SP => ld!(ops, r12w),
        PC => ld!(ops, r13w),
    }
}

pub fn store_reg(ops: &mut Assembler, r: Reg) {
    macro_rules! st {
        ($ops:expr, $r:tt) => {
            dynasm!($ops
                ; mov $r, di
            )
        };
    }

    use Reg::*;
    match r {
        AF => deconstruct_af(ops),
        BC => st!(ops, bx),
        DE => st!(ops, cx),
        HL => st!(ops, dx),
        SP => st!(ops, r12w),
        PC => st!(ops, r13w),
    }
}

/// Generate a correct AF register in di
pub fn materialize_af(ops: &mut Assembler) {
    dynasm!(ops
        ; mov [rsp + 0x01], al
        ; mov ah, [rsp + 0x02]
        ; mov al, ah
        ; and al, BYTE 0x70 as _
        ; shl al, 1
        ; and ah, BYTE 1 as _
        ; shl ah, 4
        ; or al, ah
        ; mov ah, [rsp + 0x01]
        ; mov di, ax
    );
}

/// Generate the LAHF format and the al register from AF in di
pub fn deconstruct_af(ops: &mut Assembler) {
    dynasm!(ops
        ; mov ax, di
        ; mov ah, al
        ; shr al, 1
        ; shr ah, 4
        ; and ah, BYTE 1 as _
        ; or al, ah
        ; mov [rsp + 0x02], al
        ; mov ax, di
        ; mov al, ah
    );
}

pub fn load_carry_flag(ops: &mut Assembler) {
    dynasm!(ops
        ; test [rsp + 2], 1
        ; jz >l1
        ; stc
        ; jmp >l2
        ; l1:
        ; clc
        ; l2:
    );
}

/// The ZF flag will be 1 if the condition is met
pub fn load_condition(ops: &mut Assembler, cond: Condition) {
    use Condition::*;
    match cond {
        Always => {
            dynasm!(ops
                ; cmp eax, eax
            );
        }
        Z => {
            dynasm!(ops
                ; mov ah, [rsp + 0x02]
                ; not ah
                ; test ah, 0x40
            );
        }
        Nz => {
            dynasm!(ops
                ; test BYTE [rsp + 0x02], BYTE 0x40
            );
        }
        C => {
            dynasm!(ops
                ; mov ah, [rsp + 0x02]
                ; not ah
                ; test ah, 0x01
            );
        }
        Nc => {
            dynasm!(ops
                ; test BYTE [rsp + 0x02], BYTE 0x01
            );
        }
    }
}

pub fn direct_jump(ops: &mut Assembler, target: u16, labels: &[DynamicLabel], base_addr: u16) {
    let target_idx = target.wrapping_sub(base_addr);
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

pub fn push_reg(ops: &mut Assembler, bus: &ExternalBus, reg: Reg) {
    dynasm!(ops
        ;; load_reg(ops, reg)
        ; mov [rsp + 0x00], di
        ; sub r12w, 1
        ; mov di, r12w
        ; mov sil, [rsp + 0x01]
        ;; call_write(ops, bus)
        ; sub r12w, 1
        ; mov di, r12w
        ; mov sil, [rsp + 0x00]
        ;; call_write(ops, bus)
    );
}

pub fn pop_reg(ops: &mut Assembler, bus: &ExternalBus, reg: Reg) {
    dynasm!(ops
        ; mov di, r12w
        ;; call_read(ops, bus)
        ; mov [rsp + 0x00], ah
        ; add r12w, 1
        ; mov di, r12w
        ;; call_read(ops, bus)
        ; mov [rsp + 0x01], ah
        ; add r12w, 1
        ; mov di, [rsp + 0x00]
        ;; store_reg(ops, reg)
    );
}
