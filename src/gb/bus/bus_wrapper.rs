use crate::gb::devices::Ppu;

use super::Bus;

pub struct DeviceWrapper<'a> {
    ppu: &'a mut Ppu,
}

pub struct BusWrapper<'a> {
    bus: &'a mut Bus,
    devices: DeviceWrapper<'a>,
}

impl<'a> BusWrapper<'a> {
    pub fn new(bus: &'a mut Bus, ppu: &'a mut Ppu) -> Self {
        BusWrapper {
            bus,
            devices: DeviceWrapper { ppu },
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        self.bus.read(&self.devices, addr)
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.bus.write(&self.devices, addr, val)
    }
}
