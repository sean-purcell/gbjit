#[derive(Debug, PartialEq, Hash, Copy, Clone)]
pub enum Kind {
    Bios,
    Cartridge,
    Vram,
    Cram,
    Wram,
    Oam,
    Unused,
    Io,
    Hram,
}
