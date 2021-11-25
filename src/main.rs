extern crate sdl2;
mod drawing;
mod events;
mod rendering;
mod windows;

use rendering::main_loop;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::WindowPos;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Instant;
use windows::WindowBuilder;

const MAIN_WIDTH: usize = 800;
const MAIN_HEIGHT: usize = 600;

pub fn main() -> Result<(), String> {
    // Call setup functions for sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    // Call Main Window Init from windows.rs
    let mut main_window = WindowBuilder::new(
        &video_subsystem,
        "➕Newton's Fractal➕",
        MAIN_WIDTH as u32,
        MAIN_HEIGHT as u32,
    )
    .set_position(WindowPos::Centered, WindowPos::Centered)
    .build()?;

    // Start rendering thread
    let (ttx, rx) = mpsc::channel();
    let (tx, trx) = mpsc::channel();
    let main_thread = thread::spawn(move || main_loop((ttx, trx)));

    // Init rendering ops
    let main_op = Arc::new(drawing::mandelbrot::Mandelbrot::init(
        MAIN_WIDTH as u32,
        MAIN_HEIGHT as u32,
        0,
        0,
    ));

    // Send rendering ops

    // Start the event loop, handle all events, and manage rendering ops's
    // status. Also, keep track of and print framerate.
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut now = Instant::now();
    'running: loop {
        // Handle Events
        for event in event_pump.poll_iter() {
            match event {
                // If sdl2 wants to quite or escape is pressed,
                // Then quit
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                // Default: Send event off to the event thread
                _ => {}
            }
        }
        // Handle finished rendering
    }
    Ok(())
}
