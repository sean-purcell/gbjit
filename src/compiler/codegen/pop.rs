use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _labels: &[DynamicLabel],
    _pc: u16,
    _base_addr: u16,
    bus: &ExternalBus,
) -> EpilogueDescription {
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
        deconstruct_af(ops);
    } else {
        dynasm!(ops
            ; mov di, [rsp]
            ;; store_reg(ops, reg)
        );
    }

    Default::default()
}
