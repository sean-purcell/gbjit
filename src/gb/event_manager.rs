use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::rc::Rc;

use super::CycleState;

pub type EventCycle = u64;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum EventSource {
    Ppu,
    FrameEnd,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct EventEntry {
    cycle: EventCycle,
    source: EventSource,
}

pub struct EventManager {
    cycles: Rc<CycleState>,
    events: BinaryHeap<Reverse<EventEntry>>,
}

struct EventIter(Vec<EventSource>);

impl EventManager {
    pub fn new(cycles: Rc<CycleState>) -> Self {
        EventManager {
            cycles,
            events: BinaryHeap::new(),
        }
    }

    pub fn add_event(&mut self, source: EventSource, cycle: EventCycle) {
        self.events.push(Reverse(EventEntry { cycle, source }));
        self.update_limit();
    }

    pub fn update_limit(&self) {
        let new_limit = self
            .events
            .peek()
            .map_or(std::u64::MAX, |front| front.0.cycle);
        self.cycles.set_hard_limit(new_limit);
    }

    pub fn get_events(&mut self) -> impl Iterator<Item = EventSource> {
        let mut ret = vec![];
        let current_cycle = self.cycles.cycle();
        while let Some(front) = self.events.peek().copied() {
            if front.0.cycle <= current_cycle {
                self.events.pop();
                ret.push(front.0.source);
            } else {
                break;
            }
        }
        ret.reverse();
        EventIter(ret)
    }
}

impl Iterator for EventIter {
    type Item = EventSource;

    fn next(&mut self) -> Option<EventSource> {
        self.0.pop()
    }
}
