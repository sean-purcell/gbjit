use std::path::Path;
use std::rc::Rc;

use anyhow::Error;

use crate::compiler::{CompileOptions, CycleState, ExternalBus};
use crate::cpu_state::CpuState;
use crate::executor::Executor;

pub mod bus;
pub mod devices;
mod event_manager;

use bus::{Bus, DeviceWrapper, PageId, PageStatus};
use devices::{Frame, Ppu};
use event_manager::{EventCycle, EventManager, EventSource};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct ExecutionState {
    pc: u16,
    id: PageId,
    version: u64,
}

pub struct Gb {
    cycles: Rc<CycleState>,
    cpu_state: CpuState,
    bus: Bus,
    ppu: Ppu,

    event_manager: EventManager,
    executor: Executor<PageId, Gb>,
    execution_state: Option<ExecutionState>,
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
        // TODO: Allow for halted cpu
        let pc = self.cpu_state.pc;
        let (page, data) = self.map_page(pc);
        let code = self
            .executor
            .compile(page.id, page.version, page.base_addr, data);
        let mut cpu_state = self.cpu_state;
        let cycle_state = self.cycles;
        code.enter(&mut cpu_state, self, &*cycle_state);
    }

    fn device_wrapper<'a>(&'a mut self) -> (DeviceWrapper<'a>, &'a mut Bus) {
        (DeviceWrapper::new(&mut self.ppu), &mut self.bus)
    }

    fn read(&mut self, addr: u16) -> u8 {
        let (devices, bus) = self.device_wrapper();
        bus.read(&devices, addr)
    }

    fn write(&mut self, addr: u16, val: u8) {
        let (devices, bus) = self.device_wrapper();
        bus.write(&devices, addr, val)
    }

    fn map_page(&mut self, addr: u16) -> (PageStatus, &[u8]) {
        let (devices, bus) = self.device_wrapper();
        bus.map_page(&devices, addr)
    }
}
