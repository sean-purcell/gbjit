#![allow(dead_code)]

use log::*;

pub struct Dummy([u8; 65536]);

impl Dummy {
    pub fn new() -> Self {
        Dummy([0; 65536])
    }

    pub fn read(&mut self, addr: u16) -> (u8, bool) {
        let val = self.0[addr as usize];
        debug!("Read  {:#06x?} -> {:#04x}", addr, val);
        (val, false)
    }

    pub fn write(&mut self, addr: u16, val: u8) -> bool {
        debug!("Write {:#06x?} <- {:#04x}", addr, val);
        self.0[addr as usize] = val;
        false
    }

    pub fn interrupts(&mut self, enabled: bool) -> bool {
        debug!("Interrupts enabled: {}", enabled);
        false
    }
}
