use std::rc::Rc;

use crate::compiler::CycleState;
use crate::gb::bus::Bus;

use crossbeam_channel as chan;

mod frame;

pub use frame::*;

const FRAME_TIME: u64 = 70224;

enum State {
    Hblank,
    Vblank,
    Oam,
    Render,
    Disabled,
}

pub struct Ppu {
    cycles: Rc<CycleState>,
    state: State,
    state_started: u64,

    line: u8,

    current_frame: Box<Frame>,

    sender: chan::Sender<Box<Frame>>,
}

impl State {
    fn mode(self) -> u8 {
        use Mode::*;
        match self {
            Hblank => 0,
            Vblank => 1,
            Oam => 2,
            Render => 3,
            Disabled => 1, // TODO: determine what mode actually appears during disabled
        }
    }

    fn cycles(self) -> u64 {
        match self {
            Hblank => 200,
            Vblank => 4560,
            Oam => 84,
            Render => 172,
            Disabled => 70224,
        }
    }
}

impl Ppu {
    pub fn new(cycles: Rc<CycleState>) -> (Self, chan::Receiver<Box<Frame>>) {
        let (sender, receiver) = chan::unbounded();

        let mut ppu = Ppu {
            cycles,
            state: State::Disabled,
            state_started: 0,
            enabled: false,
            line: 0,
            current_frame: Box::new(empty_frame()),
            sender,
        };

        self.update_state_cycle_limit();

        (ppu, receiver)
    }

    fn update_state(&mut self, new_state: State) {
        self.state_started += self.state.cycles();
        self.state = new_state;
        self.update_state_cycle_limit();
    }

    fn update_state_cycle_limit(&self) {
        self.cycles
            .upper_bound_hard_limit(self.state_started + self.state.cycles());
    }

    fn state_cycle_limit_hit(&self) -> bool {
        let limit = self.state_started + self.state.cycles();
        self.cycles.cycle() >= limit
    }

    pub fn process(&mut self, bus: &mut Bus) {
        if !self.state_cycle_limit_hit() {
            return;
        }

        match self.state {
            State::Hblank =>
        }
    }
}
