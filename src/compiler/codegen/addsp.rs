use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _bus: &ExternalBus,
) -> EpilogueDescription {
    let offset = parse_cmd!(inst, AddSp(offset) => offset);

    dynasm!(ops
        ; mov [rsp], dx
        ; mov dx, r12w
        ; add dl, BYTE offset as _
        ; lahf
        ; adc dh, 0
        ; mov r12w, dx
        ; mov dx, [rsp]
        ; and ah, BYTE 0x11
        ; mov [rsp + 0x02], ah
    );

    Default::default()
}
