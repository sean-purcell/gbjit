use super::util::*;

pub fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _labels: &[DynamicLabel],
    _pc: u16,
    _base_addr: u16,
    _bus: &ExternalBus,
) -> GenerateEpilogue {
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

    true
}
