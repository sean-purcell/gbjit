use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    _inst: &Instruction,
    _bus: &ExternalBus,
) -> EpilogueDescription {
    dynasm!(ops
        ; mov r12w, dx
    );

    Default::default()
}
