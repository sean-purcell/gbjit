use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _pc: u16,
    _bus: &ExternalBus,
) -> EpilogueDescription {
    let (dst, val) = parse_cmd!(inst, LdFullImm { dst, val } => (dst, val));

    dynasm!(ops
        ; mov di, WORD val as _
    );
    store_reg(ops, dst);

    Default::default()
}
