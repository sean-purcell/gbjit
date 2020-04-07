use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _pc: u16,
    bus: &ExternalBus,
) -> EpilogueDescription {
    let (inc, load) = parse_cmd!(inst, LdAddrInc { inc, load } => (inc, load));

    load_reg(ops, Reg::HL);
    if load {
        call_read(ops, bus);
        dynasm!(ops
            ; mov al, ah
        );
    } else {
        dynasm!(ops
            ; mov sil, al
        );
        call_write(ops, bus);
    }

    dynasm!(ops
        ; add dx, BYTE if inc { 1 } else { -1 }
    );
    Default::default()
}
