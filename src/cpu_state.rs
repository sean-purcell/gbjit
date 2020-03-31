#[derive(Debug, Clone)]
pub struct CpuState {
    pub af: u16,
    pub bc: u16,
    pub cd: u16,
    pub de: u16,
    pub hl: u16,
    pub sp: u16,
    pub pc: u16,
    pub intenable: bool,
    pub cycles: u64,
}
