use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _bus: &ExternalBus,
) -> EpilogueDescription {
    let reg = parse_cmd!(inst, AddHl(reg) => reg);

    dynasm!(ops
        ;; load_reg(ops, reg)
        ; mov [rsp], di
        ; add dl, [rsp + 0x00]
        ; adc dh, [rsp + 0x01]
        ; lahf
        ; and ah, BYTE 0x11
        ; and BYTE [rsp + 0x02], BYTE 0x40
        ; or BYTE [rsp + 0x02], ah
    );

    Default::default()
}
