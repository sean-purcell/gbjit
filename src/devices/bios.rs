use std::path::Path;

use log::*;

use super::memory::Rom;
use super::{Device, Error, Kind, Page, PageId};

pub struct Bios(Rom);

impl Bios {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Bios(Rom::new(path)?))
    }
}

impl Device for Bios {
    fn base_addr(&self) -> u16 {
        0
    }

    fn size(&self) -> u16 {
        256
    }

    fn map(&mut self, addr: u16) -> &mut dyn Page {
        self
    }
}

impl Page for Bios {
    fn base_addr(&self) -> u16 {
        Device::base_addr(self)
    }

    fn size(&self) -> u16 {
        Device::size(self)
    }

    fn id(&self) -> PageId {
        (Kind::Bios, 0)
    }

    fn version(&self) -> u64 {
        0
    }

    fn read(&mut self, addr: u16) -> u8 {
        (*self.0)[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        warn!("Attempted to write to BIOS {:#06x?} <- {:02x?}", addr, val);
    }

    fn read_all(&mut self) -> &[u8] {
        &*self.0
    }

    fn dirty(&self) -> bool {
        false
    }
}
