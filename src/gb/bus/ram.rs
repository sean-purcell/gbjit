use super::{Kind, Module, PageStatus};

pub struct Ram {
    mem: Vec<u8>,
    versions: Vec<u64>,
    kind: Kind,
    base_addr: u16,
    page_size: u16,
}

impl Ram {
    pub fn new(kind: Kind, base_addr: u16, size: u16, page_size: u16) -> Ram {
        Ram::new_with_data(vec![0u8; size as usize], kind, base_addr, page_size)
    }

    pub fn new_with_data<T: Into<Vec<u8>>>(
        data: T,
        kind: Kind,
        base_addr: u16,
        page_size: u16,
    ) -> Ram {
        let mem = data.into();
        let size = mem.len() as u16;

        let rem = size
            .checked_rem(page_size)
            .expect("Page size should not be 0");
        assert!(rem == 0, "Page size should be a divisor of size");

        let pages = size / page_size;
        let versions = vec![0; pages as usize];

        Ram {
            mem,
            versions,
            kind,
            base_addr,
            page_size,
        }
    }
}

impl Module for Ram {
    fn read(&mut self, addr: u16) -> u8 {
        self.mem[addr.wrapping_sub(self.base_addr) as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        let idx = addr.wrapping_sub(self.base_addr);

        let loc = &mut self.mem[idx as usize];

        if *loc != val {
            let page_idx = idx / self.page_size;
            self.versions[page_idx as usize] += 1;
            *loc = val;
        }
    }

    fn map_page(&mut self, addr: u16) -> PageStatus {
        let idx = addr.wrapping_sub(self.base_addr);
        let page_idx = idx / self.page_size;
        PageStatus {
            id: (self.kind, page_idx as u64),
            version: self.versions[page_idx as usize],
            base_addr: self.base_addr.wrapping_add(page_idx * self.page_size),
            size: self.page_size,
            fetch_key: page_idx as u64,
        }
    }

    fn read_page(&mut self, fetch_key: u64) -> &[u8] {
        let base = (fetch_key as usize) * (self.page_size as usize);
        &self.mem[base..base + self.page_size as usize]
    }
}
