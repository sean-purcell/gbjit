#![allow(dead_code)]

use std::rc::Rc;

use crate::compiler::CycleState;
use crate::gb::bus::Bus;

mod frame;

pub use frame::*;

pub const FRAME_TIME: u64 = 70224;

#[derive(Debug, Copy, Clone)]
enum Mode {
    Hblank,
    Vblank,
    Oam,
    Render,
}

#[derive(Debug, Default, Copy, Clone)]
struct Settings {
    enabled: bool,
    oam_interrupt: bool,
    vblank_interrupt: bool,
    hblank_interrupt: bool,

    scroll_xy: (u8, u8),
    compare_line: u8,
    window_xy: (u8, u8),
}

pub struct Ppu {
    cycles: Rc<CycleState>,
    mode: Mode,
    mode_started: u64,

    line: u8,

    current_frame: Box<Frame>,

    s: Settings,
}

impl Mode {
    fn id(self) -> u8 {
        use Mode::*;
        match self {
            Hblank => 0,
            Vblank => 1,
            Oam => 2,
            Render => 3,
        }
    }

    fn cycles(self) -> u64 {
        use Mode::*;
        match self {
            Hblank => 200,
            Vblank => 4560,
            Oam => 84,
            Render => 172,
        }
    }
}

impl Ppu {
    pub fn new(cycles: Rc<CycleState>) -> Self {
        let ppu = Ppu {
            cycles,
            mode: Mode::Oam,
            mode_started: 0,
            line: 0,
            current_frame: Box::new(empty_frame()),
            s: Default::default(),
        };

        // TODO: Set Oam/Vram accessibility here

        ppu.update_mode_cycle_limit();

        ppu
    }

    fn update_mode(&mut self, new_mode: Mode) {
        self.mode_started += self.mode.cycles();
        self.mode = new_mode;
        self.update_mode_cycle_limit();
    }

    fn update_mode_cycle_limit(&self) {
        self.cycles
            .upper_bound_hard_limit(self.mode_started + self.mode.cycles());
    }

    fn mode_cycle_limit_hit(&self) -> bool {
        let limit = self.mode_started + self.mode.cycles();
        self.cycles.cycle() >= limit
    }

    pub fn process(&mut self, bus: &mut Bus) {
        if !self.mode_cycle_limit_hit() {
            return;
        }

        self.end_mode(bus);
    }

    fn start_mode(&mut self, new_mode: Mode, _bus: &mut Bus) {
        self.update_mode(new_mode);

        match self.mode {
            Mode::Hblank => {
                // TODO: Unlock OAM and VRAM
            }
            Mode::Vblank => {}
            Mode::Oam => {
                // TODO: Lock OAM
            }
            Mode::Render => {
                // TODO: Lock VRAM
            }
        }
    }

    fn end_mode(&mut self, bus: &mut Bus) {
        match self.mode {
            Mode::Hblank => {
                self.line += 1;
                if self.line == 144 {
                    self.start_mode(Mode::Vblank, bus);
                } else {
                    self.start_mode(Mode::Oam, bus);
                }
            }
            Mode::Vblank => {
                self.line = 0;
                self.start_mode(Mode::Oam, bus);
            }
            Mode::Oam => {
                self.start_mode(Mode::Render, bus);
            }
            Mode::Render => {
                self.current_frame[self.line as usize] = self.render_line(bus);
                self.start_mode(Mode::Hblank, bus);
            }
        };
    }

    fn render_line(&mut self, _bus: &mut Bus) -> Scanline {
        let mut line = empty_scanline();

        for i in 0..FRAME_COLS {
            line[i] = Colour(i as u8, 255, 255);
        }

        line
    }

    pub fn scanline(&self) -> u8 {
        self.line
    }

    pub fn take_frame(&mut self) -> Option<Box<Frame>> {
        if (self.line as usize) < FRAME_ROWS {
            None
        } else {
            let mut frame = Box::new(empty_frame());
            std::mem::swap(&mut frame, &mut self.current_frame);
            Some(frame)
        }
    }
}
