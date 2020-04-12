use std::path::Path;

use log::*;
use quick_error::quick_error;

use super::memory::Rom;
use super::Error as DeviceError;
use super::{Device, Kind, Page, PageId};

quick_error! {
    #[derive(Debug)]
    pub enum Error {}
}

pub struct Cartridge {
    rom: Rom,
}

impl Cartridge {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, DeviceError> {
        Ok(Cartridge {
            rom: Rom::new(path)?,
        })
    }
}

impl Device for Cartridge {
    fn base_addr(&self) -> u16 {
        0
    }
    fn size(&self) -> u16 {
        256
    }

    fn map(&mut self, addr: u16) -> &mut dyn Page {
        // TODO:
        unimplemented!()
    }
}
