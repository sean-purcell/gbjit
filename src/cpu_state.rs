#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct CpuState {
    pub cycles: u64,
    pub sp: u16,
    pub pc: u16,
    pub af: u16,
    pub bc: u16,
    pub cd: u16,
    pub de: u16,
    pub hl: u16,
    pub intenable: bool,
}

impl CpuState {
    pub fn new() -> Self {
        Default::default()
    }
}