use super::windows::SafeTexture;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc::Receiver, Arc};
// Holds all the drawing logic, like the graph rendering and the settings display
// Also parses text and makes equation logic

pub fn main_loop(
    (main_lock, graphing_lock, sender): (Arc<SafeTexture>, Arc<SafeTexture>, Receiver<bool>),
    monitor: Arc<AtomicBool>,
) -> Result<(), String> {
    // Initialize any variables we want
    let mut nframes = 0;
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
            let mut main_texture = match main_lock.lock() {
                Ok(t) => t,
                Err(_) => return Err("The main mutex was poisoned!".to_string()),
            };
            let mut graphing_texture = match graphing_lock.lock() {
                Ok(t) => t,
                Err(_) => return Err("The main mutex was poisoned!".to_string()),
            };
            match draw_spirals(&mut main_texture, &mut graphing_texture, &mut nframes) {
                Ok(_) => (),
                Err(s) => println!("Draw spirals failed with the error: {}", s),
            };
        }
        // Once we finished using the textures, presumably
        // the main thread will update the screen and send us a message
        nframes += 1;
    }

    Ok(())
}

fn draw_spirals(
    _main_texture: &mut sdl2::render::Texture,
    streaming_texture: &mut sdl2::render::Texture,
    nframes: &mut usize,
) -> Result<(), String> {
    let main_window_size = _main_texture.query().width;
    let center = main_window_size as f64 / 2.;
    let speed = main_window_size as f64 / 500.;
    let period = main_window_size as f64 / 10.;
    let pi2 = std::f64::consts::PI * 2.;

    streaming_texture.with_lock(None, |pixels, pitch| {
        for x in 0..main_window_size {
            for y in 0..main_window_size {
                let xc = center - x as f64;
                let yc = center - y as f64;
                let r = {
                    let dist = (xc * xc + yc * yc).sqrt();
                    let rate = speed * 2.5 * *nframes as f64;
                    let cos = (pi2 * (dist - rate) / period).cos();
                    255. * 0.5 * (1.0 + cos)
                } as u8;
                let g = {
                    let dist = (xc * xc + yc * yc).sqrt();
                    // let dist = xc * xc + yc * yc;
                    let rate = speed * -0.5 * *nframes as f64;
                    let cos = (pi2 * (dist - rate) / period).sin();
                    150. * 0.5 * (1.0 + cos)
                } as u8;
                let b = {
                    let dist = (xc * xc + yc * yc).sqrt();
                    let rate = speed * *nframes as f64;
                    let sin = (pi2 * (dist - rate) / period).sin();
                    200. * 0.5 * (1.0 + sin)
                } as u8;

                let ind = y as usize * pitch + x as usize * 4;

                pixels[ind + 0] = r;
                pixels[ind + 1] = g;
                pixels[ind + 2] = b;
                pixels[ind + 3] = 255;
            }
        }
    })?;
    *nframes += 1;
    Ok(())
}
