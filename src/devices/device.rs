use super::Page;

pub trait Device {
    fn base_addr(&self) -> u16;
    fn size(&self) -> u16;

    fn map(&mut self, addr: u16) -> &mut dyn Page;
}
