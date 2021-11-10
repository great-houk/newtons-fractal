extern crate sdl2;
mod events;
mod windows;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::WindowPos;
use std::time::Instant;
use windows::GraphingWindow;

pub fn main() -> Result<(), String> {
    // Call setup functions for sdl2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // Call Main Window Init and Open Function from windows.rs
    let mut main_window = GraphingWindow::init(
        &video_subsystem,
        "Main Window",
        300,
        300,
        WindowPos::Centered,
        WindowPos::Centered,
    )?;

    // Call Settings Window Init from windows.rs

    // Call All Other Window Init Functions from windows.rs

    // Make a new thread to handle the drawing logic in logic.rs

    // Make a new thread to handle and process all events,
    // Sending the data off to the required threads, covered in events.rs

    // Start the event loop and pass the events to the proper threads,
    // As well as wait for any draw requests from the drawing thread
    let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string())?;
    'running: loop {
        let now = Instant::now();

        // Handle Events
        for event in event_pump.poll_iter() {
            match event {
                // If sdl2 wants to quite or escape is pressed,
                // Then quit
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                // If a window resizes, then we need to tell it
                Event::Window {
                    win_event: WindowEvent::Resized(wid, hei),
                    window_id: id,
                    ..
                } => {
                    if id == main_window.window().id() {
                        main_window.resized(wid as u32, hei as u32)?;
                    } else {
                        return Err("Window Resize Event Fail".to_string());
                    }
                }
                // Default: Send event off to the event thread
                _ => {}
            }
        }

        // See if there are any frames to grab
        // If so, copy and present it

        let framerate = 1000000. / now.elapsed().as_micros() as f64;
        println!("Framerate: {}", framerate);
    }

    // Triangle of messaging
    /*
                   Events with
                   Drawing Impacts
         logic.rs <------- events.rs
            \                  >
             \               /
    Updated   \            /   All Events I want to handle
    Frames     \         /
                >      /
                 main.rs

     */

    Ok(())
}
