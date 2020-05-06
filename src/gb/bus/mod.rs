use std::path::Path;

use log::*;

pub mod dummy;

mod bios;
mod cartridge;
mod error;
mod io;
mod kind;
mod module;
mod ram;
mod rom;

pub use bios::Bios;
pub use cartridge::Cartridge;
pub use error::Error;
pub use io::Io;
pub use kind::Kind;
pub use module::{Module, PageId, PageStatus};
pub use ram::Ram;
use rom::Rom;

type Oam = Ram;
type Vram = Ram;
type Wram = Ram;
type Hram = Ram;

// TODO: Fixme with mbc detection
type CartridgeRam = Ram;

pub struct Bus {
    bios: Bios,
    cart: Cartridge,
    pub vram: Vram,
    cram: CartridgeRam,
    wram: Wram,
    pub oam: Oam,
    pub io: Io,
    hram: Hram,

    bios_enabled: bool,
}

impl Bus {
    pub fn new<P: AsRef<Path>, R: AsRef<Path>>(
        bios_path: P,
        cartridge_path: R,
    ) -> Result<Self, Error> {
        Ok(Bus {
            bios: Bios::new(bios_path)?,
            cart: Cartridge::new(cartridge_path)?,
            vram: Ram::new(Kind::Vram, 0x8000, 0x2000, 0x100),
            cram: Ram::new(Kind::Cram, 0xA000, 0x2000, 0x100),
            wram: Ram::new(Kind::Wram, 0xC000, 0x2000, 0x100),
            oam: Ram::new(Kind::Oam, 0xFE00, 0xA0, 0xA0),
            io: Default::default(),
            hram: Ram::new(Kind::Hram, 0xFF80, 0x7F, 0x7F),
            bios_enabled: true,
        })
    }

    fn map_device(&mut self, addr: u16) -> Option<&mut dyn Module> {
        macro_rules! mmap {
            ($($pattern:pat => $module:ident,)*) => {
                match addr {
                    $($pattern => Some(&mut self.$module),)*
                    _ => None,
                }
            }
        }
        if self.bios_enabled && addr < 0x100 {
            return Some(&mut self.bios);
        }

        mmap! {
            0x0000..=0x7FFF => cart,
            0x8000..=0x9FFF => vram,
            0xA000..=0xBFFF => cram,
            0xC000..=0xFDFF => wram,
            0xFE00..=0xFE9F => oam,
            0xFF00..=0xFF7F => io,
            0xFF80..=0xFFFE => hram,
            0xFFFF => io,
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        let val = self.map_device(addr).map_or(0xff, |page| page.read(addr));
        val
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.map_device(addr)
            .map_or((), |page| page.write(addr, val));
    }
}
