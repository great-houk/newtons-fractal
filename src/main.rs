extern crate sdl2;
mod events;
mod rendering;
mod windows;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::WindowPos;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use windows::GraphingWindow;
use windows::Message;

const MAIN_WIDTH: usize = 800;
const MAIN_HEIGHT: usize = 600;

pub fn main() -> Result<(), String> {
    // Call setup functions for sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    // Call Main Window Init from windows.rs
    let mut main_window = GraphingWindow::init(
        &video_subsystem,
        "➕Newton's Fractal➕",
        MAIN_WIDTH,
        MAIN_HEIGHT,
        WindowPos::Centered,
        WindowPos::Centered,
    )
    .unwrap();

    // Start rendering thread

    // Init rendering ops

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
