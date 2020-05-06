use log::*;

use super::util::*;

pub(super) fn generate(
    ops: &mut Assembler,
    inst: &Instruction,
    bus: &ExternalBus,
) -> EpilogueDescription {
    let cmd = parse_cmd!(inst, Control(cmd) => cmd);

    use ControlCommand::*;
    match cmd {
        Nop => {}
        Halt | Stop => {
            dynasm!(ops
                ;; push_state(ops)
                ; mov rax, QWORD log_halt as _
                ; mov di, r13w
                ; call rax
                ;; pop_state(ops)
            );
        }
        Ccf => {
            dynasm!(ops
                ; and BYTE [rsp + 0x02], 0x70
            );
        }
        Scf => {
            dynasm!(ops
                ; or BYTE [rsp + 0x02], 0x01
            );
        }
        Di | Ei => {
            let enable = cmd == Ei;
            if enable {
                int_enable(ops);
            } else {
                int_disable(ops);
            };
        }
    };

    Default::default()
}

extern "sysv64" fn log_halt(pc: u16) {
    warn!("Executing halt/stop at {:#06x?}", pc);
}
