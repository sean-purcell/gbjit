use crate::gb::bus::Bus;

use super::*;

impl Ppu {
    pub(super) fn render_line(&mut self, bus: &mut Bus) -> Scanline {
        let mut line = empty_scanline();

        for (i, col) in line.iter_mut().enumerate() {
            *col = Colour(i as u8, 255, 255);
        }

        line
    }
}
