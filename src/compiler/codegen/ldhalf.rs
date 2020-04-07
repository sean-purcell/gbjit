use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _labels: &[DynamicLabel],
    _pc: u16,
    _base_addr: u16,
    bus: &ExternalBus,
) -> EpilogueDescription {
    let (src, dst) = parse_cmd!(inst, LdHalf { src, dst } => (src, dst));

    let load_address_param = |ops: &mut Assembler, id: HalfWordId| match id {
        RegAddr(r) => {
            load_reg(ops, r);
        }
        Addr(a) => {
            dynasm!(ops
                ; mov di, WORD a as _
            );
        }
        IoImmAddr(a) => {
            dynasm!(ops
                ; mov di, WORD (0xff00 + a as u16) as _
            );
        }
        IoRegAddr(r) => {
            load_halfreg(ops, r);
            dynasm!(ops
                ; mov di, WORD 0xff00 as _
                ; mov [rsp], ah
                ; mov BYTE [rsp + 0x01], 0
                ; add di, [rsp]
            );
        }
        _ => panic!("Non-address halfword id: {:?}", id),
    };

    use HalfWordId::*;
    match src {
        RegVal(r) => load_halfreg(ops, r),
        Imm(v) => dynasm!(ops
            ; mov ah, BYTE v as _
        ),
        c => {
            load_address_param(ops, c);
            call_read(ops, bus);
        }
    };

    match dst {
        RegVal(r) => store_halfreg(ops, r),
        Imm(_) => panic!("Storing to immediate is meaningless"),
        c => {
            dynasm!(ops
                ; mov [rsp], ah
                ; mov sil, [rsp]
            );
            load_address_param(ops, c);
            call_write(ops, bus);
        }
    };

    Default::default()
}
