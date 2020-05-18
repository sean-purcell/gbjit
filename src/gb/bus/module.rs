use super::Kind;

pub type PageId = (Kind, u64);

pub struct PageStatus {
    /// This is used as the key in the compilation cache, it should stay constant for a region
    pub id: PageId,
    /// This should change every time the data in this page changes and the page needs to be
    /// recompiled
    pub version: u64,

    pub base_addr: u16,
    pub size: u16,
}

pub trait Module {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);

    fn map_page(&mut self, addr: u16) -> (PageStatus, &[u8]);
}
