use super::util::*;

pub fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _labels: &[DynamicLabel],
    _pc: u16,
    _base_addr: u16,
    bus: &ExternalBus,
) -> GenerateEpilogue {
    let reg = parse_cmd!(inst, Push(reg) => reg);

    if reg == Reg::AF {
        // Materialize the flags register
        dynasm!(ops
            ; mov [rsp + 0x01], al
            ; mov ah, [rsp + 0x02]
            ; mov al, ah
            ; and al, BYTE 0x70 as _
            ; shl al, 1
            ; and ah, BYTE 1 as _
            ; shl ah, 4
            ; or al, ah
            ; mov [rsp + 0x00], al
        );
    } else {
        dynasm!(ops
            ;; load_reg(ops, reg)
            ; mov [rsp], di
        );
    }

    dynasm!(ops
        ; sub r12w, 1
        ; mov di, r12w
        ; mov sil, [rsp + 0x01]
        ;; call_write(ops, bus)
        ; sub r12w, 1
        ; mov di, r12w
        ; mov sil, [rsp + 0x00]
        ;; call_write(ops, bus)
    );

    true
}
