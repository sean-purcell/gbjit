use std::fs;
use std::io;
use std::ops::Deref;
use std::path::Path;

pub struct Rom {
    data: Vec<u8>,
}

impl Rom {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        Ok(Rom {
            data: fs::read(path)?,
        })
    }
}

impl Deref for Rom {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data.as_slice()
    }
}
