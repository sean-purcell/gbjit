use super::util::*;

pub fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    _labels: &[DynamicLabel],
    _pc: u16,
    _base_addr: u16,
    bus: &ExternalBus,
) -> GenerateEpilogue {
    let (src, dst) = parse_cmd!(inst, LdHalf { src, dst } => (src, dst));

    let load_address_param = |ops: &mut Assembler, id: HalfWordId| match id {
        RegAddr(r) => {
            load_reg(ops, r);
            true
        }
        Addr(a) => {
            dynasm!(ops
                ; mov di, WORD a as _
            );
            true
        }
        IoImmAddr(a) => {
            dynasm!(ops
                ; mov di, WORD (0xff00 + a as u16) as _
            );
            true
        }
        IoRegAddr(r) => {
            load_halfreg(ops, r);
            dynasm!(ops
                ; mov di, WORD 0xff00 as _
                ; mov [rsp], ah
                ; add di, [rsp]
            );
            true
        }
        _ => panic!("Non-address halfword id: {:?}", id),
    };

    use HalfWordId::*;
    let gen_load = match src {
        RegVal(r) => {
            load_halfreg(ops, r);
            false
        }
        Imm(v) => {
            dynasm!(ops
                ; mov ah, BYTE v as _
            );
            false
        }
        c => load_address_param(ops, c),
    };

    if gen_load {
        call_read(ops, bus);
    }

    let gen_store = match dst {
        RegVal(r) => {
            store_halfreg(ops, r);
            false
        }
        Imm(_) => panic!("Storing to immediate is meaningless"),
        c => load_address_param(ops, c),
    };

    if gen_store {
        dynasm!(ops
            ; mov [rsp], ah
            ; mov sil, [rsp]
        );
        call_write(ops, bus);
    }

    true
}
