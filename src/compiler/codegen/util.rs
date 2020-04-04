pub use dynasm::dynasm;
pub use dynasmrt::x64::Assembler;
pub use dynasmrt::{DynamicLabel, DynasmApi, DynasmLabelApi};

pub(super) use super::Command::*;
pub(super) use super::{ExternalBus, GenerateEpilogue, HalfReg, HalfWordId, Instruction, Reg};

macro_rules! parse_cmd {
    ($i:expr, $p:pat => $e:expr) => {
        if let $p = ($i).cmd {
            $e
        } else {
            panic!("Wrong pattern provided")
        }
    };
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
        ; mov rdi, rbp
        ; mov rax, QWORD bus.read as _
        ; call rax
        ; mov ah, al
        ;; pop_state(ops)
    );
}

pub fn call_write(ops: &mut Assembler, bus: &ExternalBus) {
    dynasm!(ops
        ;; push_state(ops)
        ; mov rdi, rbp
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
        F => panic!("F should not be used in a 8-bit load"),
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
        F => panic!("F should not be used in a 8-bit store"),
    }
}

pub fn load_reg(ops: &mut Assembler, r: Reg) {
    macro_rules! ld {
        ($ops:expr, $r:tt) => {
            dynasm!($ops
                ; mov si, $r
            )
        };
    }

    use Reg::*;
    match r {
        AF => panic!("AF should not be used in a 16-bit load"),
        BC => ld!(ops, bx),
        DE => ld!(ops, cx),
        HL => ld!(ops, dx),
        SP => ld!(ops, r12w),
    }
}

pub fn store_reg(ops: &mut Assembler, r: Reg) {
    macro_rules! st {
        ($ops:expr, $r:tt) => {
            dynasm!($ops
                ; mov $r, si
            )
        };
    }

    use Reg::*;
    match r {
        AF => panic!("AF should not be used in a 16-bit store"),
        BC => st!(ops, bx),
        DE => st!(ops, cx),
        HL => st!(ops, dx),
        SP => st!(ops, r12w),
    }
}
