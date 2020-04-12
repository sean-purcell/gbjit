use std::io;
use std::path::Path;

mod bios;
pub mod cartridge;
mod device;
mod error;
mod kind;
pub mod memory;
mod page;

pub use device::Device;
pub use error::Error;
pub use kind::Kind;
pub use page::{Page, PageId};

type Bios = memory::Rom;
type Cartridge = memory::Rom;

pub struct Devices {
    bios: Bios,
    cart: Cartridge,

    bios_enabled: bool,
}

impl Devices {
    pub fn new<P: AsRef<Path>, R: AsRef<Path>>(
        bios_path: P,
        cartridge_path: R,
    ) -> Result<Self, Error> {
        unimplemented!()
    }
}
