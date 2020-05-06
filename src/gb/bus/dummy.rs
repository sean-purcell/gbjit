#![allow(dead_code)]

use log::*;

pub struct Dummy([u8; 65536]);

impl Default for Dummy {
    fn default() -> Self {
        Self::new()
    }
}

impl Dummy {
    pub fn new() -> Self {
        Dummy([0; 65536])
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        let val = self.0[addr as usize];
        debug!("Read  {:#06x?} -> {:#04x}", addr, val);
        val
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        debug!("Write {:#06x?} <- {:#04x}", addr, val);
        self.0[addr as usize] = val;
    }
}
