use derive_more::From;

use super::{Kind, Module, PageStatus, Ram};

#[derive(From)]
pub struct Wram(Ram);

impl Wram {
    pub fn new() -> Self {
        Ram::new(Kind::Wram, 0xC000, 0x2000, 0x100).into()
    }
}

impl Module for Wram {
    fn read(&mut self, addr: u16) -> u8 {
        self.0.read(addr & !0x2000)
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.0.write(addr & !0x2000, val)
    }

    fn map_page(&mut self, addr: u16) -> (PageStatus, &[u8]) {
        let (mut ps, data) = self.0.map_page(addr & !0x2000);
        if addr >= 0xE000 {
            ps.id.1 += 0x20;
            ps.base_addr += 0x2000;
        }
        (ps, data)
    }
}
