use std::path::Path;

use log::*;
use quick_error::quick_error;

use super::Error as BusError;
use super::{Kind, Module, PageStatus, Rom};

quick_error! {
    #[derive(Debug)]
    pub enum Error {}
}

pub struct Cartridge {
    rom: Rom,
}

impl Cartridge {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, BusError> {
        Ok(Cartridge {
            rom: Rom::new(path)?,
        })
    }
}

impl Module for Cartridge {
    fn read(&mut self, addr: u16) -> u8 {
        // TODO: implement MBC
        self.rom[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        // TODO: implement MBC
        warn!(
            "Attempted to write to Cartridge {:#06x?} <- {:02x?}",
            addr, val
        );
    }

    fn map_page(&mut self, addr: u16) -> PageStatus {
        // TODO: implement MBC
        let idx = addr / 0x4000;
        let base_addr = idx * 0x4000;
        PageStatus {
            id: (Kind::Cartridge, idx as u64),
            version: 0,
            base_addr,
            size: 0x4000,
            fetch_key: base_addr as _,
        }
    }

    fn read_page(&mut self, fetch_key: u64) -> &[u8] {
        let idx = fetch_key as usize;
        &self.rom[idx..idx + 0x4000]
    }
}
