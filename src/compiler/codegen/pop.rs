use super::util::*;

pub fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _labels: &[DynamicLabel],
    _pc: u16,
    _base_addr: u16,
    bus: &ExternalBus,
) -> GenerateEpilogue {
    let reg = parse_cmd!(inst, Pop(reg) => reg);

    dynasm!(ops
        ; mov di, r12w
        ;; call_read(ops, bus)
        ; mov [rsp + 0x00], ah
        ; add r12w, 1
        ; mov di, r12w
        ;; call_read(ops, bus)
        ; mov [rsp + 0x01], ah
        ; add r12w, 1
    );
    if reg == Reg::AF {
        // Construct the LAHF form from the flags register
        dynasm!(ops
            ; mov ah, [rsp + 0x00]
            ; mov al, ah
            ; shr al, 1
            ; shr ah, 4
            ; and ah, BYTE 1 as _
            ; or al, ah
            ; mov [rsp + 0x02], al
            ; mov al, [rsp + 0x01]
        );
    } else {
        dynasm!(ops
            ; mov di, [rsp]
            ;; store_reg(ops, reg)
        );
    }

    true
}
