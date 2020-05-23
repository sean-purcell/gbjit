// FIXME: Remove once used
#![allow(dead_code)]

use std::cell::UnsafeCell;
use std::cmp::min;
use std::fmt;

/// Wrap the current cycle count and the cycle count at which the cpu must return back to the
/// system.  There are two types of limits: hard limits, and interrupt limits.  Hard limits are
/// used for hard system requirements, such as needing to draw a scanline or take user input.
/// Interrupt limits are cycle counts at which we need to generate an interrupt, and so should only
/// cause an exit if interrupts are enabled.  A combined limit is maintained internally so that
/// when interrupts are enabled, the cpu only has one value to compare against.
///
/// Unsafe cells are used internally to provide raw pointers to the assembly.  It is safe to
/// maintain other immutable references in other areas, such as generating the right value when
/// reading timers.
#[derive(Default)]
pub struct CycleState {
    cycle: UnsafeCell<u64>,
    hard_limit: UnsafeCell<u64>,
    interrupt_limit: UnsafeCell<u64>,
    combined_limit: UnsafeCell<u64>,
}

#[repr(C)]
pub(super) struct RawCycleState {
    cycle: *mut u64,
    int_disabled_limit: *mut u64,
    int_enabled_limit: *mut u64,
}

impl CycleState {
    pub fn new() -> Self {
        let state: CycleState = Default::default();
        state.set_hard_limit(std::u64::MAX);
        state.set_interrupt_limit(std::u64::MAX);
        state
    }

    fn update(&self) {
        let min_val = min(get(&self.hard_limit), get(&self.interrupt_limit));
        set(&self.combined_limit, min_val)
    }

    pub fn advance(&self, count: u64) {
        set(&self.cycle, get(&self.cycle) + count)
    }

    /// Get the current cycle count.
    pub fn cycle(&self) -> u64 {
        get(&self.cycle)
    }

    /// Set the hard cycle limit.
    pub fn set_hard_limit(&self, val: u64) {
        set(&self.hard_limit, val);
        self.update();
    }

    pub fn force_stop(&self) {
        set(&self.hard_limit, 0);
        self.update();
    }

    /// Update the hard limit to the minimum of the current value and the provided value
    pub fn upper_bound_hard_limit(&self, val: u64) {
        self.set_hard_limit(min(val, get(&self.hard_limit)));
    }

    /// Set the intterupt cycle limit.
    pub fn set_interrupt_limit(&self, val: u64) {
        set(&self.interrupt_limit, val);
        self.update();
    }

    pub(super) fn raw(&self) -> RawCycleState {
        RawCycleState {
            cycle: self.cycle.get(),
            int_disabled_limit: self.hard_limit.get(),
            int_enabled_limit: self.combined_limit.get(),
        }
    }
}

impl fmt::Debug for CycleState {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("CycleState")
            .field("cycle", &get(&self.cycle))
            .field("hard_limit", &get(&self.hard_limit))
            .field("interrupt_limit", &get(&self.interrupt_limit))
            .field("combined_limit", &get(&self.combined_limit))
            .finish()
    }
}

fn get(cell: &UnsafeCell<u64>) -> u64 {
    unsafe { *cell.get() }
}

fn set(cell: &UnsafeCell<u64>, val: u64) {
    unsafe { *cell.get() = val }
}
