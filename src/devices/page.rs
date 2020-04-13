use super::Kind;

pub type PageId = (Kind, u64);

pub trait Page {
    fn base_addr(&self) -> u16;
    fn size(&self) -> u16;

    fn id(&self) -> PageId;
    /// This should change every time the data inside changes
    fn version(&self) -> u64;

    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);

    // TODO: default implementation?
    fn read_all(&mut self) -> &[u8];
}
