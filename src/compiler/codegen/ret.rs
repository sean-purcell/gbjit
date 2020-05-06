use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    bus: &ExternalBus,
) -> EpilogueDescription {
    let (condition, intenable) =
        parse_cmd!(inst, Ret { condition, intenable } => (condition, intenable));

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

    dynasm!(ops
        ; mov di, r12w
        ;; call_read(ops, bus)
        ; mov [rsp + 0x00], ah
        ; inc r12w
        ; mov di, r12w
        ;; call_read(ops, bus)
        ; mov [rsp + 0x01], ah
        ; inc r12w
        ; mov di, [rsp + 0x00]
    );

    if intenable {
        int_enable(ops);
    }

    let target = JumpDescription::Dynamic;

    EpilogueDescription::Jump { target, skip_label }
}
