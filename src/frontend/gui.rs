use std::error::Error as StdError;
use std::mem;
use std::time::{Duration, Instant};

use glium::{
    buffer::{Buffer, BufferMode, BufferType},
    framebuffer::SimpleFrameBuffer,
    glutin::{
        dpi::LogicalSize,
        event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
    texture::Texture2d,
    uniforms::MagnifySamplerFilter,
    BlitTarget, Display, Rect, Surface,
};
use log::*;

use crate::{
    executor::ExecutorOptions,
    gb::{
        devices::ppu::{Frame, FRAME_COLS, FRAME_ROWS},
        Gb,
    },
    Args,
};

type GlColour = (u8, u8, u8);

pub fn run(args: Args) -> Result<(), Box<dyn StdError>> {
    let mut gb = Gb::new(&args.bios, &args.rom, ExecutorOptions::new(&args))?;

    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(
            args.screen_dimensions.0,
            args.screen_dimensions.1,
        ))
        .with_resizable(false)
        .with_title("JIT Gameboy Emulator");

    let cb = ContextBuilder::new();

    let display = Display::new(wb, cb, &event_loop)?;

    let pixels = display.get_framebuffer_dimensions();

    let texture = Texture2d::empty(&display, FRAME_COLS as _, FRAME_ROWS as _)?;

    let buffer: Buffer<[GlColour]> = Buffer::empty_unsized(
        &display,
        BufferType::ArrayBuffer,
        FRAME_ROWS * FRAME_COLS * mem::size_of::<GlColour>(),
        BufferMode::Persistent,
    )?;

    let mut last_frame = Instant::now();
    let frame_time = Duration::from_secs_f64(0.01674270629);

    let mut run_frame = move |last_frame: &mut Instant| {
        let start = Instant::now();
        debug!("Simulating GB");
        let frame = gb
            .run_frame()
            .expect("Experienced error while producing frame");
        debug!("Simulation finished");

        let data = transcribe_frame(&*frame);
        let slice = buffer.as_slice();
        slice.write(&data[..]);
        texture.main_level().raw_upload_from_pixel_buffer(
            slice,
            0..(FRAME_COLS as u32),
            0..(FRAME_ROWS as u32),
            0..1,
        );

        let surface = display.draw();
        let frame_buffer =
            SimpleFrameBuffer::new(&display, &texture).expect("Failed to create framebuffer");
        surface.blit_from_simple_framebuffer(
            &frame_buffer,
            &Rect {
                left: 0,
                bottom: 0,
                width: FRAME_COLS as u32,
                height: FRAME_ROWS as u32,
            },
            &BlitTarget {
                left: 0,
                bottom: 0,
                width: pixels.0 as i32,
                height: pixels.1 as i32,
            },
            MagnifySamplerFilter::Nearest,
        );
        surface.finish().expect("Surface failed to draw");

        *last_frame += frame_time;
        debug!("Frame took {:#?}", Instant::now() - start);
    };

    event_loop.run(move |event, _, flow| {
        debug!("Event: {:?}", event);

        match event {
            Event::NewEvents(StartCause::ResumeTimeReached {
                start: _,
                requested_resume: _,
            }) => {
                if !args.wait {
                    run_frame(&mut last_frame)
                }
            }
            #[allow(deprecated)]
            Event::WindowEvent {
                window_id: _,
                event:
                    WindowEvent::KeyboardInput {
                        device_id: _,
                        input:
                            KeyboardInput {
                                scancode: _,
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::N),
                                modifiers: _,
                            },
                        is_synthetic: false,
                    },
            } => {
                if args.wait {
                    run_frame(&mut last_frame)
                }
            }
            _ => {}
        }

        if !args.wait {
            *flow = ControlFlow::WaitUntil(last_frame + frame_time);
        } else {
            *flow = ControlFlow::Wait;
        }
    });
}

pub fn transcribe_frame(frame: &Frame) -> [GlColour; FRAME_ROWS * FRAME_COLS] {
    let mut result = [(0, 0, 0); FRAME_ROWS * FRAME_COLS];
    for (src, dst) in frame
        .iter()
        .rev()
        .flat_map(|x| x.iter())
        .zip(result.iter_mut())
    {
        *dst = (src.0, src.1, src.2);
    }
    result
}
