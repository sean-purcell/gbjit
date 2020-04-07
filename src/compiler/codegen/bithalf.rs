use super::util::*;

fn generate_shift(ops: &mut Assembler, cmd: BitCommand, set_zero: bool) {
    macro_rules! emit_shift {
        ($ops:expr, $o:tt, $set_zero:expr) => {
            {
                dynasm!($ops
                    ; mov [rsp], al
                    ; mov al, ah
                    ; $o al, 1
                    ; lahf
                    ; and al, BYTE 1
                );
                if $set_zero {
                    dynasm!($ops
                        ; cmp al, BYTE 0
                        ; jne >noset
                        ; or al, BYTE 0x40
                        ; noset:
                    );
                }
                dynasm!(ops
                    ; mov [rsp + 0x02], ah
                    ; mov ah, al
                    ; mov al, [rsp]
                );
            }
        }
    }

    let sz = set_zero;

    use BitCommand::*;
    match cmd {
        Rl | Rr => load_carry_flag(ops),
        _ => {}
    };

    match cmd {
        Rlc => emit_shift!(ops, rol, sz),
        Rl => emit_shift!(ops, rcl, sz),
        Rrc => emit_shift!(ops, ror, sz),
        Rr => emit_shift!(ops, rcr, sz),
        Sla => emit_shift!(ops, sal, sz),
        Sra => emit_shift!(ops, sar, sz),
        Srl => emit_shift!(ops, shr, sz),
        _ => unreachable!(),
    };
}

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _labels: &[DynamicLabel],
    _pc: u16,
    _base_addr: u16,
    bus: &ExternalBus,
) -> EpilogueDescription {
    let (cmd, op) = parse_cmd!(inst, BitHalf { cmd, op } => (cmd, op));

    load_location(ops, bus, op);

    use BitCommand::*;
    let store = match cmd {
        Rlc | Rl | Rrc | Rr | Sla | Sra | Srl => {
            generate_shift(ops, cmd, inst.size() == 2);
            true
        }
        Swap => {
            dynasm!(ops
                ; ror ah, BYTE 4
                ; cmp ah, BYTE 0
                ; je >set
                ; mov BYTE [rsp + 0x02], BYTE 0x00
                ; jmp >end
                ; set:
                ; mov BYTE [rsp + 0x02], BYTE 0x40
                ; end:
            );
            true
        }
        Bit(b) => {
            dynasm!(ops
                ; test ah, BYTE (1u8 << b) as _
                ; jz >zero
                ; mov ah, 0x50
                ; jmp >end
                ; zero:
                ; mov ah, 0x10
                ; end:
                ; and BYTE [rsp + 0x02], BYTE 1
                ; or [rsp + 0x02], ah
            );
            false
        }
        Set(b) => {
            dynasm!(ops
                ; or ah, BYTE (1u8 << b) as _
            );
            true
        }
        Res(b) => {
            dynasm!(ops
                ; and ah, BYTE !(1u8 << b) as _
            );
            true
        }
    };

    if store {
        store_location(ops, bus, op);
    }

    Default::default()
}
