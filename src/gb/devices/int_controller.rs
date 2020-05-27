use crate::compiler::CycleState;

pub struct IntController {
    cyles: Rc<CycleState>,
    enable: u8,
    request: u8,
}
