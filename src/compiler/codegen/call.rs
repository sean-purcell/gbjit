use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    pc: u16,
    bus: &ExternalBus,
) -> EpilogueDescription {
    let (target, condition) = parse_cmd!(inst, Call { target, condition } => (target, condition));

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

    let next_pc = pc.wrapping_add(inst.size());
    dynasm!(ops
        ; dec r12w
        ; mov di, r12w
        ; mov sil, BYTE (next_pc >> 8) as _
        ;; call_write(ops, bus)
        ; dec r12w
        ; mov di, r12w
        ; mov sil, BYTE (next_pc & 0xff) as _
        ;; call_write(ops, bus)
    );

    let target = JumpDescription::Static(target);

    EpilogueDescription::Jump { target, skip_label }
}
