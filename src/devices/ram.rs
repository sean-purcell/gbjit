use super::memory;

use super::Page as TPage;
use super::{Device, Kind, PageId};

pub struct Ram {
    pages: Vec<Page>,
    base_addr: u16,
    size: u16,
    page_size: u16,
}

struct Page {
    mem: memory::Ram,
    version: u64,
    base_addr: u16,
    size: u16,
}

impl Ram {
    pub fn new(base_addr: u16, size: u16, page_size: u16) -> Ram {
        let rem = size
            .checked_div(page_size)
            .expect("Page size should not be 0");
        assert!(rem == 0, "Page size should be a divisor of size");
        let pages = (0..size)
            .step_by(page_size as usize)
            .map(|offset| Page::new(page_size, offset + base_addr))
            .collect();
        Ram {
            pages,
            base_addr,
            size,
            page_size,
        }
    }
}

impl Device for Ram {
    fn base_addr(&self) -> u16 {
        self.base_addr
    }
    fn size(&self) -> u16 {
        self.size
    }

    fn map(&mut self, addr: u16) -> &mut dyn TPage {
        let offset = addr - self.base_addr;
        &mut self.pages[(offset / self.page_size) as usize]
    }
}

impl Page {
    fn new(size: u16, base_addr: u16) -> Page {
        Page {
            mem: memory::Ram::new(size as usize),
            version: 0,
            base_addr,
            size,
        }
    }
}

impl TPage for Page {
    fn base_addr(&self) -> u16 {
        self.base_addr
    }

    fn size(&self) -> u16 {
        self.size
    }

    fn id(&self) -> PageId {
        (Kind::Ram, self.base_addr as _)
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn read(&mut self, addr: u16) -> u8 {
        let offset = addr - self.base_addr();
        (*self.mem)[offset as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.version += 1;
        let offset = addr - self.base_addr();
        (*self.mem)[offset as usize] = val;
    }

    fn read_all(&mut self) -> &[u8] {
        &*self.mem
    }
}
