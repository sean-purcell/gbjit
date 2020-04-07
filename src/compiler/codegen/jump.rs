use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _labels: &[DynamicLabel],
    pc: u16,
    _base_addr: u16,
    _bus: &ExternalBus,
) -> EpilogueDescription {
    let (target, condition) = parse_cmd!(inst, Jump { target, condition } => (target, condition));

    let skip_label = if condition != Condition::Always {
        let label = ops.new_dynamic_label();
        load_condition(ops, condition);
        dynasm!(ops
            ; jnz =>label
        );
        Some(label)
    } else {
        None
    };

    use JumpTarget::*;
    let jump_description = match target {
        Absolute(t) => JumpDescription::Static(t),
        Hl => {
            dynasm!(ops
                ; mov di, dx
            );
            JumpDescription::Dynamic
        }
        Relative(offset) => {
            let target_pc = pc.wrapping_add(inst.size()).wrapping_add(offset as u16);
            JumpDescription::Static(target_pc)
        }
    };
    EpilogueDescription::Jump {
        target: jump_description,
        skip_label,
    }
}
