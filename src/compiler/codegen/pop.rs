use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    bus: &ExternalBus,
) -> EpilogueDescription {
    let reg = parse_cmd!(inst, Pop(reg) => reg);

    pop_reg(ops, bus, reg);

    Default::default()
}
