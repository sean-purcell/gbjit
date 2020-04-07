use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _labels: &[DynamicLabel],
    _pc: u16,
    _base_addr: u16,
    bus: &ExternalBus,
) -> EpilogueDescription {
    let (loc, inc) = parse_cmd!(inst, IncDecHalf { loc, inc } => (loc, inc));

    load_location(ops, bus, loc);

    if inc {
        dynasm!(ops
            ; inc ah
        );
    } else {
        dynasm!(ops
            ; dec ah
        );
    }

    dynasm!(ops
        ; mov [rsp], ah
        ; lahf
        ; and BYTE [rsp + 0x02], BYTE 0x01
        ; or [rsp + 0x02], ah
        ; mov ah, [rsp]
    );

    store_location(ops, bus, loc);

    Default::default()
}
