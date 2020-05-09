pub const FRAME_COLS: usize = 160;
pub const FRAME_ROWS: usize = 144;

#[derive(Default, Debug, Copy, Clone)]
pub struct Colour(pub u8, pub u8, pub u8);

pub type Scanline = [Colour; FRAME_COLS];

pub type Frame = [Scanline; FRAME_ROWS];

pub fn empty_scanline() -> Scanline {
    [Default::default(); FRAME_COLS]
}

pub fn empty_frame() -> Frame {
    [empty_scanline(); FRAME_ROWS]
}
