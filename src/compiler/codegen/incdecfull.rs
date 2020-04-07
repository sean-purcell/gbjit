use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _pc: u16,
    _bus: &ExternalBus,
) -> EpilogueDescription {
    let (reg, inc) = parse_cmd!(inst, IncDecFull { reg, inc } => (reg, inc));

    load_reg(ops, reg);
    if inc {
        dynasm!(ops
            ; inc di
        );
    } else {
        dynasm!(ops
            ; dec di
        );
    }
    store_reg(ops, reg);

    Default::default()
}
