use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _pc: u16,
    bus: &ExternalBus,
) -> EpilogueDescription {
    let reg = parse_cmd!(inst, Push(reg) => reg);

    push_reg(ops, bus, reg);

    Default::default()
}
