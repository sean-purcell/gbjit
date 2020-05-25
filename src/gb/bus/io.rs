use crate::gb::devices::Ppu;

use super::{DeviceWrapper, Kind, PageStatus};

trait Device {
    fn read(&mut self, offset: u8) -> u8;
    fn write(&mut self, offset: u8, val: u8);
}

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

    fn read_mem(&mut self, offset: u8) -> u8 {
        self.mem[offset as usize]
    }

    fn write_mem(&mut self, offset: u8, val: u8) {
        let ro_mask = ro_map(offset);
        let current_val = self.mem[offset as usize];
        let new_val = (current_val & ro_mask) | (val & !ro_mask);
        self.mem[offset as usize] = new_val;
    }

    fn map_device<'a: 'd, 'b: 'd, 'c: 'd, 'd>(
        &'a mut self,
        devices: &'b mut DeviceWrapper<'c>,
        offset: u8,
    ) -> &'d mut dyn Device {
        match offset {
            0x40..=0x45 | 0x47..=0x49 => devices.ppu,
            _ => self,
        }
    }

    pub fn read(&mut self, devices: &mut DeviceWrapper<'_>, addr: u16) -> u8 {
        let offset = addr as u8;
        self.map_device(devices, offset).read(offset)
    }

    pub fn write(&mut self, devices: &mut DeviceWrapper<'_>, addr: u16, val: u8) {
        let offset = addr as u8;
        self.map_device(devices, offset).write(offset, val)
    }

    #[allow(dead_code)]
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
        }
    }
}

fn ro_map(offset: u8) -> u8 {
    match offset {
        _ => 0x00,
    }
}

macro_rules! impl_device_fwd {
    ($t:ty, $r:ident, $w:ident) => {
        impl Device for $t {
            fn read(&mut self, offset: u8) -> u8 {
                let res = <$t>::$r(self, offset);
                log::trace!(
                    "IO read from {:3}, 0xff{:02x} => {:02x}",
                    stringify!($t),
                    offset,
                    res
                );
                res
            }

            fn write(&mut self, offset: u8, val: u8) {
                log::trace!(
                    "IO write to  {:3}, 0xff{:02x} <= {:02x}",
                    stringify!($t),
                    offset,
                    val
                );
                <$t>::$w(self, offset, val)
            }
        }
    };

    ($t:ty) => {
        impl_device_fwd!($t, read, write);
    };
}

impl_device_fwd!(Io, read_mem, write_mem);
impl_device_fwd!(Ppu);
