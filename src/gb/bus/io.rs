use super::{DeviceWrapper, Kind, PageStatus};

#[derive(Debug)]
pub struct Io {
    mem: Vec<u8>,
}

impl Io {
    pub fn new() -> Self {
        Io {
            mem: vec![0u8; 256],
        }
    }

    pub fn read<'a>(&mut self, _devices: &DeviceWrapper<'a>, addr: u16) -> u8 {
        self.mem[(addr - 0xff00) as usize]
    }

    pub fn write<'a>(&mut self, _devices: &DeviceWrapper<'a>, addr: u16, val: u8) {
        let offset = addr as u8;
        let ro_mask = ro_map(offset);
        let current_val = self.mem[offset as usize];
        let new_val = (current_val & ro_mask) | (val & !ro_mask);
        self.mem[offset as usize] = new_val;
    }

    pub fn map_page<'a>(&mut self, _devices: &DeviceWrapper<'a>, addr: u16) -> PageStatus {
        // Because the IO pages change so often and are usually cyclical, do pages of size 1.
        // If anyone is crazy enough to execute in this region, put the current value in id so that
        // we don't throw away old versions of "pages".
        let offset = addr as u64;
        let val = self.mem[offset as usize] as u64;
        PageStatus {
            id: (Kind::Io, (offset << 8) | val),
            version: 0,
            base_addr: addr,
            size: 1,
            fetch_key: offset,
        }
    }

    pub fn read_page<'a>(&mut self, _devices: &DeviceWrapper<'a>, fetch_key: u64) -> &[u8] {
        let idx = fetch_key as usize;
        &self.mem[idx..idx + 1]
    }
}

fn ro_map(offset: u8) -> u8 {
    match offset {
        _ => 0x00,
    }
}
