use std::io;

use quick_error::quick_error;

use super::cartridge;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        IoError(err: io::Error) {
            cause(err)
            from()
        }
        CartridgeError(err: cartridge::Error) {
            cause(err)
            from()
        }
    }
}
