use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    pc: u16,
    bus: &ExternalBus,
) -> EpilogueDescription {
    let target = parse_cmd!(inst, Rst(target) => target);

    let next_pc = pc.wrapping_add(inst.size());
    push_static(ops, bus, next_pc);

    let target = JumpDescription::Static(target as u16);

    EpilogueDescription::Jump {
        target,
        skip_label: None,
    }
}
