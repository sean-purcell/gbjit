use std::path::Path;

use log::*;

use super::{Error, Kind, Module, PageStatus, Rom};

pub struct Bios(Rom);

impl Bios {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Bios(Rom::new(path)?))
    }
}

impl Module for Bios {
    fn read(&mut self, addr: u16) -> u8 {
        (*self.0)[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        warn!("Attempted to write to BIOS {:#06x?} <- {:02x?}", addr, val);
    }

    fn map_page(&mut self, _addr: u16) -> (PageStatus, &[u8]) {
        (
            PageStatus {
                id: (Kind::Bios, 0),
                version: 0,
                base_addr: 0,
                size: 256,
            },
            &*self.0,
        )
    }
}
