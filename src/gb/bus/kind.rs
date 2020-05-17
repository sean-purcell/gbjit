#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
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
