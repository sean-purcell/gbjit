use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    bus: &ExternalBus,
) -> EpilogueDescription {
    let target = parse_cmd!(inst, Rst(target) => target);

    dynasm!(ops
        ; add r13w, WORD inst.size() as _
        ;; push_reg(ops, bus, Reg::PC)
        ; sub r13w, WORD inst.size() as _
    );

    let target = JumpDescription::Static(target as u16);

    EpilogueDescription::Jump {
        target,
        skip_label: None,
    }
}
