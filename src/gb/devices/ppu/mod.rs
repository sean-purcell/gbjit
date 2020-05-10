#![allow(dead_code)]

use std::collections::VecDeque;
use std::rc::Rc;

use crate::compiler::CycleState;
use crate::gb::bus::Bus;

use super::EventCycle;

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
    frame_started: u64,

    line: u8,

    current_frame: Box<Frame>,
    completed_frames: VecDeque<Box<Frame>>,

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
    pub fn new(cycles: Rc<CycleState>) -> (Self, EventCycle) {
        let current_cycle = cycles.cycle();
        let ppu = Ppu {
            cycles,
            mode: Mode::Oam,
            mode_started: current_cycle,
            frame_started: current_cycle,
            line: 0,
            current_frame: Box::new(empty_frame()),
            completed_frames: VecDeque::new(),
            s: Default::default(),
        };

        let limit = ppu.mode_cycle_limit();

        (ppu, limit)
    }

    fn mode_cycle_limit(&self) -> u64 {
        self.mode_started + self.mode.cycles()
    }

    fn mode_cycle_limit_hit(&self) -> bool {
        self.cycles.cycle() >= self.mode_cycle_limit()
    }

    pub fn process(&mut self, bus: &mut Bus) -> EventCycle {
        if !self.mode_cycle_limit_hit() {
            panic!("Process should not have been called before the event limit was hit")
        }

        self.end_mode(bus);

        self.mode_cycle_limit()
    }

    fn start_mode(&mut self, new_mode: Mode, _bus: &mut Bus) {
        self.mode_started += self.mode.cycles();
        self.mode = new_mode;

        match self.mode {
            Mode::Hblank => {
                // TODO: Unlock OAM and VRAM
            }
            Mode::Vblank => {
                let mut frame = Box::new(empty_frame());
                std::mem::swap(&mut frame, &mut self.current_frame);
                self.completed_frames.push_back(frame);
            }
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
                self.frame_started = self.mode_started;
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
        self.completed_frames.pop_front()
    }

    pub fn next_frame_end(&self) -> EventCycle {
        self.frame_started + FRAME_TIME
    }
}
