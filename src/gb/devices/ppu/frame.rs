const FRAME_COLS: usize = 160;
const FRAME_ROWS: usize = 144;

#[derive(Default, Debug, Copy, Clone)]
pub struct Colour(u8, u8, u8);

pub type Scanline = [Colour; FRAME_COLS];

pub type Frame = [Scanline; FRAME_ROWS];

pub fn empty_frame() -> Frame {
    [[Default::default(); FRAME_COLS]; FRAME_ROWS]
}
