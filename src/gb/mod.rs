use std::path::Path;
use std::rc::Rc;

use anyhow::Error;

use crate::compiler::{CompileOptions, CycleState, ExternalBus};
use crate::cpu_state::CpuState;
use crate::executor::Executor;

pub mod bus;
pub mod devices;
mod event_manager;

use bus::{Bus, DeviceWrapper, PageId};
use devices::{Frame, Ppu};
use event_manager::{EventCycle, EventManager, EventSource};

pub struct Gb {
    cycles: Rc<CycleState>,
    cpu_state: CpuState,
    bus: Bus,
    ppu: Ppu,

    event_manager: EventManager,
    executor: Executor<PageId, Gb>,
}

impl Gb {
    pub fn new<P: AsRef<Path>, R: AsRef<Path>>(
        bios_path: P,
        cartridge_path: R,
        options: CompileOptions,
    ) -> Result<Self, Error> {
        let cycles = Rc::new(CycleState::new());
        let cpu_state = CpuState::new();
        let bus = Bus::new(bios_path, cartridge_path)?;
        let (ppu, ppu_cycle) = Ppu::new(cycles.clone());
        let mut event_manager = EventManager::new(cycles.clone());
        let executor = Executor::new(
            ExternalBus {
                read: Gb::read,
                write: Gb::write,
            },
            options,
        )?;

        event_manager.add_event(EventSource::Ppu, ppu_cycle);

        Ok(Gb {
            cycles,
            cpu_state,
            bus,
            ppu,
            event_manager,
            executor,
        })
    }

    pub fn run_frame(&mut self) -> Box<Frame> {
        self.event_manager
            .add_event(EventSource::FrameEnd, self.ppu.next_frame_end());

        let mut frame_ended = false;
        while !frame_ended {
            self.cpu_exec();
            for source in self.event_manager.get_events() {
                use EventSource::*;
                match source {
                    Ppu => {
                        let next = self.ppu.process(&mut self.bus);
                        self.event_manager.add_event(Ppu, next);
                    }
                    FrameEnd => frame_ended = true,
                }
            }
        }

        self.ppu
            .take_frame()
            .expect("A frame should be complete by now")
    }

    fn cpu_exec(&mut self) {
        self.cycles.advance(4)
    }

    fn read(&mut self, addr: u16) -> u8 {
        let devices = DeviceWrapper::new(&mut self.ppu);
        self.bus.read(&devices, addr)
    }

    fn write(&mut self, addr: u16, val: u8) {
        let devices = DeviceWrapper::new(&mut self.ppu);
        self.bus.write(&devices, addr, val)
    }
}
