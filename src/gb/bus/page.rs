use super::Kind;

pub type PageId = (Kind, u64);

pub struct PageStatus {
    /// This is used as the key in the compilation cache, it should stay constant for a region
    pub id: PageId,
    /// This should change every time the data in this page changes and the page needs to be
    /// recompiled
    pub version: u64,

    /// This should be passed as the parameter to the Device's read_page function
    pub fetch_key: usize,
}
