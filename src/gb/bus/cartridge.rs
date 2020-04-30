use std::path::Path;

use log::*;
use quick_error::quick_error;

use super::memory::Rom;
use super::Error as DeviceError;
use super::Page as TPage;
use super::{Device, Kind, PageId};

quick_error! {
    #[derive(Debug)]
    pub enum Error {}
}

pub struct Cartridge {
    pages: Vec<Page>,
}

struct Page {
    rom: Rom,
    offset: u64,
}

impl Cartridge {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, DeviceError> {
        let base_rom = Rom::new(path)?;

        let pages = (0..2)
            .map(|idx| {
                let offset = idx * 0x4000;
                Page {
                    rom: Rom::new_from_slice(&(*base_rom)[offset..offset + 0x4000]),
                    offset: offset as u64,
                }
            })
            .collect();
        Ok(Cartridge { pages })
    }
}

impl Device for Cartridge {
    fn base_addr(&self) -> u16 {
        0
    }
    fn size(&self) -> u16 {
        0x8000
    }

    fn map(&mut self, addr: u16) -> &mut dyn TPage {
        let idx = addr / 0x4000;
        &mut self.pages[idx as usize]
    }
}

impl TPage for Page {
    fn base_addr(&self) -> u16 {
        if self.offset == 0 {
            0
        } else {
            0x4000
        }
    }

    fn size(&self) -> u16 {
        0x4000
    }

    fn id(&self) -> PageId {
        (Kind::Cartridge, self.offset)
    }

    fn version(&self) -> u64 {
        0
    }

    fn read(&mut self, addr: u16) -> u8 {
        let offset = addr - self.base_addr();
        (*self.rom)[offset as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        // TODO: implement MBC's and fix this
        warn!(
            "Attempted to write to Cartridge {:#06x?} <- {:02x?}",
            addr, val
        );
    }

    fn read_all(&mut self) -> &[u8] {
        &*self.rom
    }
}
