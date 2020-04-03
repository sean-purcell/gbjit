use dynasm::dynasm;
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
        ; push rax
        ; push rcx
        ; push rdx
        ; sub rsp, 8
    );
}

pub fn pop_state(ops: &mut Assembler) {
    // TODO: add r11 when we start using that
    dynasm!(ops
        ; add rsp, 8
        ; pop rax
        ; pop rcx
        ; pop rdx
    );
}

pub fn load_halfreg(ops: &mut Assembler, r: HalfReg) {
    macro_rules! ld {
        ($ops:expr, $r:tt) => {
            dynasm!($ops
                ; mov dil, $r
            )
        };

        ($ops:expr, $rh:tt, $rl:tt) => {
            dynasm!($ops
                ; xchg $rh, $rl
                ; mov dil, $rl
                ; xchg $rl, $rh
            )
        };
    }

    use HalfReg::*;
    match r {
        A => ld!(ops, al),
        B => ld!(ops, bh, bl),
        C => ld!(ops, bl),
        D => ld!(ops, ch, cl),
        E => ld!(ops, cl),
        H => ld!(ops, dh, dl),
        L => ld!(ops, dl),
        F => panic!("F should not be used in a 8-bit ld"),
    }
}
