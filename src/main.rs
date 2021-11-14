extern crate sdl2;
mod events;
mod logic;
mod windows;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::WindowPos;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use windows::GraphingWindow;
use windows::Message;

pub fn main() -> Result<(), String> {
    // Call setup functions for sdl2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // Call Main Window Init and Open Function from windows.rs
    let mut main_window = GraphingWindow::init(
        &video_subsystem,
        "Main Window",
        500,
        500,
        WindowPos::Centered,
        WindowPos::Centered,
    )?;
    let main_data = main_window.get_textures();

    // Call Settings Window Init from windows.rs

    // Call All Other Window Init Functions from windows.rs

    // Set up a thread monitor to make sure thread it's always alive
    let mut monitor = Arc::new(AtomicBool::new(true));
    let mut control = Arc::downgrade(&monitor);

    // Make a new thread to handle the drawing logic in logic.rs
    let (mut width, mut height) = main_window.raw.canvas.output_size()?;
    let mut main_thread = Some(thread::spawn(move || {
        logic::main_loop(main_data, monitor, width, height)
    }));

    // Make a new thread to handle and process all events,
    // Sending the data off to the required threads, covered in events.rs

    // Start the event loop and pass the events to the proper threads,
    // As well as wait for any draw requests from the drawing thread
    let mut event_pump = sdl_context.event_pump()?;
    let mut now = Instant::now();
    'running: loop {
        // Check on thread status, and respond accordingly
        match control.upgrade() {
            // Logic thread is alive
            Some(_) => {
                // See if there are any frames to grab
                // If so, copy and present it
                if main_window.present()? {
                    let time_elapsed = Instant::elapsed(&now).as_micros();
                    now = Instant::now();
                    let fr = 1_000_000 / time_elapsed;
                    println!("Framerate: {}", fr);
                }
            }
            // Logic thread is dead
            None => {
                println!("Logic thread died");
                // Get result from thread and
                // process the result
                main_thread
                    .take()
                    .expect("Logic thread isn't there")
                    .join()
                    .expect("Logic thread panicked")?;
                // Remake it
                monitor = Arc::new(AtomicBool::new(true));
                control = Arc::downgrade(&monitor);
                let main_data = main_window.get_textures();
                main_thread = Some(thread::spawn(move || {
                    logic::main_loop(main_data, monitor, width, height)
                }));
            }
        }

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
                    main_window.send(Message::Quit);
                    main_thread
                        .take()
                        .expect("Logic thread isn't there")
                        .join()
                        .expect("Logic thread panicked")?;
                    break 'running;
                }
                // If a window resizes, then we need to tell it
                Event::Window {
                    win_event: WindowEvent::Resized(wid, hei),
                    window_id: id,
                    ..
                } => {
                    if id == main_window.window().id() {
                        let (w, h) = main_window.resized(wid as u32, hei as u32)?;
                        width = w;
                        height = h;
                    } else {
                        return Err("Window Resize Event Fail".to_string());
                    }
                }
                // Default: Send event off to the event thread
                _ => {}
            }
        }
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
