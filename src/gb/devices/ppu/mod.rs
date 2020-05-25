#![allow(dead_code)]

use std::collections::VecDeque;
use std::rc::Rc;

use crate::compiler::CycleState;
use crate::gb::bus::Bus;

use super::EventCycle;

mod frame;
mod render;

pub use frame::*;

pub const FRAME_TIME: u64 = 70224;

#[derive(Debug, Copy, Clone)]
enum Mode {
    Hblank,
    Vblank,
    Oam,
    Render,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Default)]
struct BwPalette(u8);

impl From<u8> for BwPalette {
    fn from(v: u8) -> Self {
        BwPalette(v)
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct Settings {
    enabled: bool,

    window_tmap: bool,
    window_en: bool,
    tile_data: bool,
    bg_tmap: bool,
    obj_size: bool,
    obj_en: bool,
    bg_en: bool,

    coincidence_interrupt: bool,
    oam_interrupt: bool,
    vblank_interrupt: bool,
    hblank_interrupt: bool,

    scroll_xy: (u8, u8),
    compare_line: u8,
    window_xy: (u8, u8),

    bg_palette: BwPalette,
    o0_palette: BwPalette,
    o1_palette: BwPalette,
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

    pub fn scanline(&self) -> u8 {
        match self.mode {
            Mode::Hblank | Mode::Oam | Mode::Render => self.line,
            Mode::Vblank => {
                let mode_cycles = self.cycles.cycle() - self.mode_started;
                144 + (mode_cycles / 456) as u8
            }
        }
    }

    pub fn take_frame(&mut self) -> Option<Box<Frame>> {
        self.completed_frames.pop_front()
    }

    pub fn next_frame_end(&self) -> EventCycle {
        self.frame_started + FRAME_TIME
    }

    pub fn read(&mut self, offset: u8) -> u8 {
        macro_rules! write_bitfield {
            { $( $idx:expr => $field:expr, )* } => {
                (
                $(
                    to_flag($field, $idx) |
                )*
                0)
            }
        }
        match offset {
            0x40 => {
                write_bitfield! {
                    7 => self.s.enabled,
                    6 => self.s.window_tmap,
                    5 => self.s.window_en,
                    4 => self.s.tile_data,
                    3 => self.s.bg_tmap,
                    2 => self.s.obj_size,
                    1 => self.s.obj_en,
                    0 => self.s.bg_en,
                }
            }
            0x41 => {
                self.mode.id()
                    | write_bitfield! {
                        6 => self.s.coincidence_interrupt,
                        5 => self.s.oam_interrupt,
                        4 => self.s.vblank_interrupt,
                        3 => self.s.hblank_interrupt,
                        2 => self.s.compare_line == self.scanline(),
                    }
            }
            0x44 => self.scanline(),
            0x47 => self.s.bg_palette.0,
            0x48 => self.s.o0_palette.0,
            0x49 => self.s.o1_palette.0,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, offset: u8, val: u8) {
        macro_rules! read_bitfield {
            { $val:expr, $( $idx:expr => $field:expr, )* } => {{
                $(
                    $field = from_flag($val, $idx);
                )*
            }}
        }
        match offset {
            0x40 => {
                read_bitfield! {
                    val,
                    7 => self.s.enabled,
                    6 => self.s.window_tmap,
                    5 => self.s.window_en,
                    4 => self.s.tile_data,
                    3 => self.s.bg_tmap,
                    2 => self.s.obj_size,
                    1 => self.s.obj_en,
                    0 => self.s.bg_en,
                }
            }
            0x41 => {
                read_bitfield! {
                    val,
                    6 => self.s.coincidence_interrupt,
                    5 => self.s.oam_interrupt,
                    4 => self.s.vblank_interrupt,
                    3 => self.s.hblank_interrupt,
                }
            }
            0x47 => self.s.bg_palette = val.into(),
            0x48 => self.s.o0_palette = val.into(),
            0x49 => self.s.o1_palette = val.into(),
            0x44 => log::warn!(
                "Attempted to write {:02x} to RO PPU reg 0xff{:02x}",
                val,
                offset
            ),
            _ => unreachable!(),
        }
    }
}

fn to_flag(val: bool, idx: usize) -> u8 {
    if val {
        1u8 << idx
    } else {
        0
    }
}

fn from_flag(val: u8, idx: usize) -> bool {
    (val & (1u8 << idx)) != 0
}
