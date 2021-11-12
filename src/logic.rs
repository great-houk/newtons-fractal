use super::windows::SafeTexture;
use sdl2::render::TextureQuery;
use std::alloc::{alloc, dealloc, Layout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc::Receiver, Arc};
use std::thread::{spawn, JoinHandle};
// Holds all the drawing logic, like the graph rendering and the settings display
// Also parses text and makes equation logic

pub fn main_loop(
    (main_lock, graphing_lock, sender): (Arc<SafeTexture>, Arc<SafeTexture>, Receiver<bool>),
    monitor: Arc<AtomicBool>,
    width: u32,
    height: u32,
) -> Result<(), String> {
    // Initialize any variables we want
    let mut nframes = 0;
    let cores = num_cpus::get();
    let pixels = {
        let layout =
            Layout::array::<u8>((width * height * 4) as usize).map_err(|e| e.to_string())?;
        let mut ptr = unsafe { alloc(layout) };
        ptr as &mut [u8]
    };
    // Start main loop
    while monitor.load(Ordering::Relaxed) {
        // Wait for the window to give up control on the textures
        match sender.recv() {
            Ok(true) => (),
            Ok(false) => return Ok(()),
            Err(_) => {
                return Err("Something happened with the transmitter in main window!".to_string())
            }
        };
        // Grab the texture's lock again, and use them
        {
            let main_texture = match main_lock.lock() {
                Ok(t) => t,
                Err(_) => return Err("The main mutex was poisoned!".to_string()),
            };
            let mut graphing_texture = match graphing_lock.lock() {
                Ok(t) => t,
                Err(_) => return Err("The main mutex was poisoned!".to_string()),
            };
        }
        // Once we finished using the textures, presumably
        // the main thread will update the screen and send us a message
        nframes += 1;
    }

    Ok(())
}

fn draw_spirals_thread(
    slice: &mut [u8],
    data: &((u32, u32), f64, f64, f64, f64),
    nframes: usize,
    ind: usize,
    pitch: usize,
) {
    for i in 0..slice.len() / 4 {
        let j = i + ind;
        let (x, y) = {
            let x = j % pitch;
            let y = j / pitch;
            (x as u32, y as u32)
        };
        let (r, g, b) = draw_spirals_fn(data, nframes, x, y);
        slice[i + 0] = r;
        slice[i + 1] = g;
        slice[i + 2] = b;
        slice[i + 3] = 255;
    }
}

fn draw_spirals_data(width: u32, height: u32) -> (f64, f64, f64, f64) {
    let center = width as f64 / 2.;
    let speed = width as f64 / 500.;
    let period = width as f64 / 10.;
    let pi2 = std::f64::consts::PI * 2.;
    (center, speed, period, pi2)
}

fn draw_spirals_fn(
    (_, center, speed, period, pi2): &((u32, u32), f64, f64, f64, f64),
    nframes: usize,
    x: u32,
    y: u32,
) -> (u8, u8, u8) {
    let xc = center - x as f64;
    let yc = center - y as f64;
    let r = {
        let dist = (xc * xc + yc * yc).sqrt();
        let rate = speed * 2.5 * nframes as f64;
        let cos = (pi2 * (dist - rate) / period).cos();
        255. * 0.5 * (1.0 + cos)
    } as u8;
    let g = {
        let dist = (xc * xc + yc * yc).sqrt();
        // let dist = xc * xc + yc * yc;
        let rate = speed * -0.5 * nframes as f64;
        let cos = (pi2 * (dist - rate) / period).sin();
        150. * 0.5 * (1.0 + cos)
    } as u8;
    let b = {
        let dist = (xc * xc + yc * yc).sqrt();
        let rate = speed * nframes as f64;
        let sin = (pi2 * (dist - rate) / period).sin();
        200. * 0.5 * (1.0 + sin)
    } as u8;

    (r, g, b)
}
