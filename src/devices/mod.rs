use std::path::Path;

use log::*;

pub mod memory;

mod bios;
mod cartridge;
mod device;
mod error;
mod kind;
mod page;
mod ram;

pub use bios::Bios;
pub use cartridge::Cartridge;
pub use device::Device;
pub use error::Error;
pub use kind::Kind;
pub use page::{Page, PageId};
pub use ram::Ram;

pub struct Devices {
    bios: Bios,
    cart: Cartridge,

    ram: Ram,

    bios_enabled: bool,
}

impl Devices {
    pub fn new<P: AsRef<Path>, R: AsRef<Path>>(
        bios_path: P,
        cartridge_path: R,
    ) -> Result<Self, Error> {
        Ok(Devices {
            bios: Bios::new(bios_path)?,
            cart: Cartridge::new(cartridge_path)?,
            ram: Ram::new(0x8000, 0x8000, 0x100),
            bios_enabled: true,
        })
    }

    fn map_addr(&mut self, addr: u16) -> Option<&mut dyn Device> {
        if self.bios_enabled && addr < 0x100 {
            Some(&mut self.bios)
        } else if addr < 0x8000 {
            Some(&mut self.cart)
        } else {
            Some(&mut self.ram)
        }
    }

    fn map_page(&mut self, addr: u16) -> Option<&mut dyn Page> {
        self.map_addr(addr).map(|device| device.map(addr))
    }

    pub fn read(&mut self, addr: u16) -> (u8, bool) {
        let val = self.map_page(addr).map_or(0xff, |page| page.read(addr));
        (val, false)
    }

    pub fn write(&mut self, addr: u16, val: u8) -> bool {
        self.map_page(addr).map_or((), |page| page.write(addr, val));
        false
    }

    pub fn interrupts(&mut self, enabled: bool) -> bool {
        debug!("Interrupts enabled: {}", enabled);
        false
    }
}
