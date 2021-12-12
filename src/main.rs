extern crate sdl2;
mod drawing;
mod events;
mod rendering;
mod windows;

use events::MainEvent;
use rendering::{main_loop, ThreadMessage};
use sdl2::video::WindowPos;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Instant;
use windows::WindowBuilder;

const MAIN_WIDTH: usize = 600;
const MAIN_HEIGHT: usize = 600;

pub fn main() -> Result<(), String> {
    // Call setup functions for sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let event_system = sdl_context.event().unwrap();
    event_system.register_custom_event::<MainEvent>().unwrap();
    // Call Main Window Init from windows.rs
    let main_window = Arc::new(Mutex::new(
        WindowBuilder::new(
            &video_subsystem,
            "➕Newton's Fractal➕",
            MAIN_WIDTH as u32,
            MAIN_HEIGHT as u32,
            |a, b| (a, b),
        )
        .set_position(WindowPos::Centered, WindowPos::Centered)
        .set_resizable(true)
        .build()?,
    ));

    // Init rendering ops
    let main_op = drawing::Mandelbrot::init(main_window.clone());

    // Start rendering thread
    let (tx, rx) = mpsc::channel();
    let rendering_transmitter = event_system.event_sender();
    let main_thread = thread::spawn(move || main_loop(rendering_transmitter, rx));

    // Init event watcher
    let mut event_handler =
        events::EventHandler::init(&sdl_context, vec![main_window], vec![main_op.clone()])?;

    // Send rendering ops
    tx.send(ThreadMessage::StartOp(main_op.clone())).unwrap();

    // Start the event loop, handle all events, and manage rendering ops's
    // status. Also, keep track of and print framerate.
    let mut now = Instant::now();
    loop {
        // Handle Events
        let events = event_handler.handle_events();
        for event in events {
            // println!("Event: {}", event);
            match event {
                MainEvent::Quit(result) => {
                    tx.send(ThreadMessage::Quit).unwrap();
                    main_thread
                        .join()
                        .expect("rendering thread panicked")
                        .unwrap();
                    return result;
                }
                MainEvent::RenderOpFinish(op) => {
                    let op = op.read().unwrap();
                    let window = op.get_window();
                    let mut window_mut = window.lock().unwrap();
                    window_mut.present(op.get_present_buffer(), *op.get_rect());
                    // Framerate
                    println_framerate(&mut now);
                }
                MainEvent::RenderOpStart(op) => {
                    tx.send(ThreadMessage::StartOp(op)).unwrap();
                }
            }
        }
    }
}

fn println_framerate(instant: &mut Instant) {
    let time_elapsed = Instant::elapsed(instant).as_micros();
    *instant = Instant::now();
    let fr;
    if time_elapsed == 0 {
        fr = u128::MAX;
    } else {
        fr = 1_000_000 / time_elapsed;
    }
    if fr > 0 {
        println!("Framerate: {}", fr);
    } else {
        println!("Time Elapsed: {}", time_elapsed / 1000);
    }
}
