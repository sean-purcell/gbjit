use std::path::Path;
use std::rc::Rc;

use anyhow::Error;

use crate::compiler::{CycleState, ExternalBus};
use crate::cpu_state::CpuState;
use crate::executor::{Executor, ExecutorOptions};

pub mod bus;
pub mod devices;
mod event_manager;

use bus::{Bus, DeviceWrapper, PageId, PageStatus};
use devices::{Frame, Ppu};
use event_manager::{EventCycle, EventManager, EventSource};

pub struct Gb {
    cycles: Rc<CycleState>,
    cpu_state: CpuState,
    components: Components,

    event_manager: EventManager,
    executor: Executor<PageId, Components>,
}

struct Components {
    cycles: Rc<CycleState>,
    bus: Bus,
    ppu: Ppu,
    execution_state: Option<ExecutionState>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct ExecutionState {
    pc: u16,
    id: PageId,
    version: u64,
}

impl Gb {
    pub fn new<P: AsRef<Path>, R: AsRef<Path>>(
        bios_path: P,
        cartridge_path: R,
        options: ExecutorOptions,
    ) -> Result<Self, Error> {
        let cycles = Rc::new(CycleState::new());
        let cpu_state = CpuState::new();
        let bus = Bus::new(bios_path, cartridge_path)?;
        let (ppu, ppu_cycle) = Ppu::new(cycles.clone());
        let mut event_manager = EventManager::new(cycles.clone());
        let executor = Executor::new(
            ExternalBus {
                read: Components::read,
                write: Components::write,
            },
            options,
        )?;
        let execution_state = None;

        event_manager.add_event(EventSource::Ppu, ppu_cycle);

        Ok(Gb {
            cycles: cycles.clone(),
            cpu_state,
            components: Components {
                cycles,
                bus,
                ppu,
                execution_state,
            },
            event_manager,
            executor,
        })
    }

    pub fn run_frame(&mut self) -> Result<Box<Frame>, Error> {
        self.event_manager
            .add_event(EventSource::FrameEnd, self.components.ppu.next_frame_end());

        let mut frame_ended = false;
        while !frame_ended {
            self.cpu_exec()?;
            for source in self.event_manager.get_events() {
                use EventSource::*;
                match source {
                    Ppu => {
                        let next = self.components.ppu.process(&mut self.components.bus);
                        self.event_manager.add_event(Ppu, next);
                    }
                    FrameEnd => frame_ended = true,
                }
            }
        }

        Ok(self
            .components
            .ppu
            .take_frame()
            .expect("Frame should be complete"))
    }

    fn cpu_exec(&mut self) -> Result<(), Error> {
        // TODO: Allow for halted cpu
        let (page, data) = self.components.map_page(self.cpu_state.pc);
        let code = self
            .executor
            .compile(page.id, page.version, page.base_addr, data)?;
        self.components.execution_state = Some(ExecutionState {
            pc: self.cpu_state.pc,
            id: page.id,
            version: page.version,
        });
        code.enter(&mut self.cpu_state, &mut self.components, &self.cycles);
        self.components.execution_state.take();
        Ok(())
    }
}

impl Components {
    fn device_wrapper(&mut self) -> (DeviceWrapper<'_>, &mut Bus) {
        (DeviceWrapper::new(&mut self.ppu), &mut self.bus)
    }

    fn read(&mut self, addr: u16) -> u8 {
        let (devices, bus) = self.device_wrapper();
        bus.read(&devices, addr)
    }

    fn do_write(&mut self, addr: u16, val: u8) {
        let (devices, bus) = self.device_wrapper();
        bus.write(&devices, addr, val)
    }

    fn map_page(&mut self, addr: u16) -> (PageStatus, &[u8]) {
        let (devices, bus) = self.device_wrapper();
        bus.map_page(&devices, addr)
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.do_write(addr, val);
        // Check if the page we're executing has been remapped
        let state = self.execution_state.unwrap();
        let (page, _) = self.map_page(state.pc);
        if (page.id, page.version) != (state.id, state.version) {
            self.cycles.force_stop();
        }
    }
}
