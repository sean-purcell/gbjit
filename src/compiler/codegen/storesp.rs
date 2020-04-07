use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _labels: &[DynamicLabel],
    _pc: u16,
    _base_addr: u16,
    bus: &ExternalBus,
) -> EpilogueDescription {
    let addr = parse_cmd!(inst, StoreSp { addr } => addr);

    dynasm!(ops
        ; mov di, WORD addr as _
        ; mov sil, r12b
        ;; call_write(ops, bus)
        ; mov di, WORD (addr + 1) as _
        ; mov si, r12w
        ; shr si, 8
        ;; call_write(ops, bus)
    );

    Default::default()
}
