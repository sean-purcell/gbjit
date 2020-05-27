use crate::gb::bus::{Bus, Module};

use super::*;

impl Ppu {
    pub(super) fn render_line(&self, bus: &mut Bus) -> Scanline {
        if !self.s.enabled {
            return white_line();
        }

        let bg = self.render_background(bus);

        bg
    }

    fn render_background(&self, bus: &mut Bus) -> Scanline {
        let s = &self.s;

        if !s.bg_en {
            return white_line();
        }

        let vram = &mut bus.vram;

        let mut line = empty_scanline();

        let tmap = s.bg_tmap.val();
        let tdata = s.tile_data;

        let y = s.scroll_xy.1.wrapping_add(self.line);

        for (i, px) in line.iter_mut().enumerate() {
            let x = s.scroll_xy.0.wrapping_add(i as u8);

            let tile_xy = (x / 8, y / 8);

            let tile_idx = (tile_xy.0 as u16) + (tile_xy.1 as u16) * 32;
            let tile_val = vram.read(tmap.wrapping_add(tile_idx));
            let tile_addr = tdata.map(tile_val);

            let (col, row) = (x % 8, y % 8);

            let addr = tile_addr.wrapping_add(row as u16 * 2);
            let b0 = vram.read(addr);
            let b1 = vram.read(addr + 1);

            let colour_idx = if b0 & (1u8 << col) != 0 { 1 } else { 0 }
                | if b1 & (1u8 << col) != 0 { 2 } else { 0 };

            *px = s.bg_palette.map(colour_idx);
        }

        line
    }
}

fn white_line() -> Scanline {
    let mut line = empty_scanline();

    for col in line.iter_mut() {
        *col = Colour(255, 255, 255);
    }

    line
}
