use super::util::*;

fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _labels: &[DynamicLabel],
    _pc: u16,
    _base_addr: u16,
    _bus: &ExternalBus,
) -> GenerateEpilogue {
    let (src, dst) = parse_cmd!(inst, LdHalf { src, dst } => (src, dst));

    use HalfWordId::*;
    let load = match src {
        RegVal(r) => load_halfreg(ops, r),
        _ => unreachable!(),
    };

    true
}
