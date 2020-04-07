use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _labels: &[DynamicLabel],
    _pc: u16,
    _base_addr: u16,
    bus: &ExternalBus,
) -> EpilogueDescription {
    let reg = parse_cmd!(inst, Push(reg) => reg);

    if reg == Reg::AF {
        materialize_af(ops);
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

    Default::default()
}
