use std::fmt;

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct CpuState {
    pub sp: u16,
    pub pc: u16,
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub intenable: bool,
}

impl CpuState {
    pub fn new() -> Self {
        Default::default()
    }
}

impl fmt::Display for CpuState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let flags = (self.af >> 8) as u8;
        let fc = |b, c| if (flags & (1u8 << b)) != 0u8 { c } else { '-' };
        f.write_fmt(format_args!(
            "A: {:02x}, F: {}{}{}{}, BC: {:04x}, DE: {:04x}, HL: {:04x}, SP: {:04x}",
            self.af as u8,
            fc(6, 'Z'),
            fc(5, 'N'),
            fc(4, 'H'),
            fc(0, 'C'),
            self.bc,
            self.de,
            self.hl,
            self.sp,
        ))
    }
}
