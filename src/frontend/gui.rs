use std::error::Error as StdError;

use glium::backend::glutin::DisplayCreationError;
use glium::glutin::{
    dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder, ContextBuilder,
};
use glium::Display;

use super::super::Args;

pub fn run(args: &Args) -> Result<(), Box<dyn StdError>> {
    let mut event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1280, 1152))
        .with_resizable(false)
        .with_title("JIT Gameboy Emulator");

    let cb = ContextBuilder::new();

    let display = Display::new(wb, cb, &event_loop)?;

    loop {}

    Ok(())
}
