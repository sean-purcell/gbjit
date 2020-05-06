use super::{Module, PageStatus};

#[derive(Debug, Default)]
pub struct Io {
    mem: Vec<u8>,
}

impl Io {
    pub fn new() -> Self {
        Io {
            mem: vec![0u8; 256],
        }
    }
}

impl Module for Io {
    fn read(&mut self, addr: u16) -> u8 {
        unimplemented!()
    }

    fn write(&mut self, addr: u16, val: u8) {
        unimplemented!()
    }

    fn map_page(&mut self, addr: u16) -> PageStatus {
        unimplemented!()
    }

    fn read_page(&mut self, fetch_key: u64) -> &[u8] {
        unimplemented!()
    }
}
