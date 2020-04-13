use std::ops::{Deref, DerefMut};

pub struct Ram {
    data: Vec<u8>,
}

impl Ram {
    pub fn new(size: usize) -> Self {
        Ram {
            data: vec![0; size],
        }
    }
}

impl Deref for Ram {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data.as_slice()
    }
}

impl DerefMut for Ram {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.as_mut_slice()
    }
}
