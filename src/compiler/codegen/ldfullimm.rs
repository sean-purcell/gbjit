use super::util::*;

pub fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _labels: &[DynamicLabel],
    _pc: u16,
    _base_addr: u16,
    _bus: &ExternalBus,
) -> GenerateEpilogue {
    let (dst, val) = parse_cmd!(inst, LdFullImm { dst, val } => (dst, val));

    dynasm!(ops
        ; mov di, WORD val as _
    );
    store_reg(ops, dst);

    true
}
