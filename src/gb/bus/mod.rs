use std::path::Path;

use crate::gb::devices::Ppu;

pub mod dummy;

mod bios;
mod bus_wrapper;
mod cartridge;
mod error;
mod io;
mod kind;
mod module;
mod ram;
mod rom;
mod wram;

pub use bios::Bios;
pub use bus_wrapper::BusWrapper;
pub use cartridge::Cartridge;
pub use error::Error;
pub use io::Io;
pub use kind::Kind;
pub use module::{Module, PageId, PageStatus};
pub use ram::Ram;
use rom::Rom;
use wram::Wram;

type Oam = Ram;
type Vram = Ram;
type Hram = Ram;

// TODO: Fixme with mbc detection
type CartridgeRam = Ram;

type Unused = Ram;

pub struct Bus {
    bios: Bios,
    cart: Cartridge,
    pub vram: Vram,
    cram: CartridgeRam,
    wram: Wram,
    pub oam: Oam,
    unused: Unused,
    pub io: Io,
    hram: Hram,

    bios_enabled: bool,
}

pub struct DeviceWrapper<'a> {
    ppu: &'a mut Ppu,
}

enum MapResult<'a> {
    Memory(&'a mut dyn Module),
    Io(&'a mut Io),
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
            wram: Wram::new(),
            oam: Ram::new(Kind::Oam, 0xFE00, 0xA0, 0xA0),
            unused: Ram::new_with_data(vec![0xff; 0x60], Kind::Unused, 0xFEA0, 0x60),
            io: Io::new(),
            hram: Ram::new(Kind::Hram, 0xFF80, 0x7F, 0x7F),
            bios_enabled: true,
        })
    }

    fn map_device<'a>(&'a mut self, addr: u16) -> MapResult<'a> {
        macro_rules! mmap {
            ($($pattern:pat => $module:ident,)*) => {
                match addr {
                    $($pattern => MapResult::Memory(&mut self.$module),)*
                    _ => MapResult::Io(&mut self.io),
                }
            }
        }
        if self.bios_enabled && addr < 0x100 {
            return MapResult::Memory(&mut self.bios);
        }

        mmap! {
            0x0000..=0x7FFF => cart,
            0x8000..=0x9FFF => vram,
            0xA000..=0xBFFF => cram,
            0xC000..=0xFDFF => wram,
            0xFE00..=0xFE9F => oam,
            0xFEA0..=0xFEFF => unused,
            // 0xFF00..=0xFF7F => io,
            0xFF80..=0xFFFE => hram,
            // 0xFFFF => io,
        }
    }

    pub fn read(&mut self, devices: &mut DeviceWrapper<'_>, addr: u16) -> u8 {
        match self.map_device(addr) {
            MapResult::Memory(m) => m.read(addr),
            MapResult::Io(io) => io.read(devices, addr),
        }
    }

    pub fn write(&mut self, devices: &mut DeviceWrapper<'_>, addr: u16, val: u8) {
        match self.map_device(addr) {
            MapResult::Memory(m) => m.write(addr, val),
            MapResult::Io(io) => io.write(devices, addr, val),
        }
    }

    pub fn map_page(&mut self, _devices: &mut DeviceWrapper<'_>, addr: u16) -> (PageStatus, &[u8]) {
        match self.map_device(addr) {
            MapResult::Memory(m) => m.map_page(addr),
            MapResult::Io(_io) => panic!("Mapping IO not yet supported"),
        }
    }
}

impl<'a> DeviceWrapper<'a> {
    pub fn new(ppu: &'a mut Ppu) -> Self {
        DeviceWrapper { ppu }
    }
}
