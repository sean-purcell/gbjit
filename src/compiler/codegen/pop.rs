use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _pc: u16,
    bus: &ExternalBus,
) -> EpilogueDescription {
    let reg = parse_cmd!(inst, Pop(reg) => reg);

    pop_reg(ops, bus, reg);

    Default::default()
}
