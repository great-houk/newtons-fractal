extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;

const MAIN_WINDOW_SIZE: u32 = 500;

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Newton's Fractal", MAIN_WINDOW_SIZE, MAIN_WINDOW_SIZE)
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let mut main_texture = texture_creator
        .create_texture_target(
            texture_creator.default_pixel_format(),
            MAIN_WINDOW_SIZE,
            MAIN_WINDOW_SIZE,
        )
        .unwrap_or_else(|error| panic!("Failed to make the main texture! {}", error));
    let mut streaming_texture = texture_creator
        .create_texture_streaming(
            sdl2::pixels::PixelFormatEnum::ABGR8888,
            MAIN_WINDOW_SIZE,
            MAIN_WINDOW_SIZE,
        )
        .unwrap_or_else(|error| panic!("Failed to make the streaming texture! {}", error));

    let mut nframes = 0;
    let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string())?;
    'running: loop {
        let now = Instant::now();

        if !handle_events(&mut event_pump) {
            break 'running;
        }

        // Run code
        draw_spirals(
            &mut canvas,
            &mut main_texture,
            &mut streaming_texture,
            &mut nframes,
        )?;

        // Present Canvas
        canvas.present();

        let framerate = 1000000. / now.elapsed().as_micros() as f64;
        println!("Framerate: {}", framerate);
    }

    Ok(())
}

fn handle_events(event_pump: &mut sdl2::EventPump) -> bool {
    for event in event_pump.poll_iter() {
        match event {
            // If sdl2 wants to quite or escape is pressed,
            // Then quit
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => return false,
            // Default
            _ => {}
        }
    }
    true
}

fn draw_spirals(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    _main_texture: &mut sdl2::render::Texture,
    streaming_texture: &mut sdl2::render::Texture,
    nframes: &mut usize,
) -> Result<(), String> {
    const CENTER: f64 = MAIN_WINDOW_SIZE as f64 / 2.;
    const SPEED: f64 = MAIN_WINDOW_SIZE as f64 / 500.;
    const PERIOD: f64 = MAIN_WINDOW_SIZE as f64 / 10.;
    const PI2: f64 = std::f64::consts::PI * 2.;

    streaming_texture.with_lock(None, |pixels, pitch| {
        for x in 0..MAIN_WINDOW_SIZE {
            for y in 0..MAIN_WINDOW_SIZE {
                let xc = CENTER - x as f64;
                let yc = CENTER - y as f64;
                let r = {
                    let dist = (xc * xc + yc * yc).sqrt();
                    let rate = SPEED * 2.5 * *nframes as f64;
                    let cos = (PI2 * (dist - rate) / PERIOD).cos();
                    255. * 0.5 * (1.0 + cos)
                } as u8;
                let g = {
                    let dist = (xc * xc + yc * yc).sqrt();
                    // let dist = xc * xc + yc * yc;
                    let rate = SPEED * -0.5 * *nframes as f64;
                    let cos = (PI2 * (dist - rate) / PERIOD).sin();
                    150. * 0.5 * (1.0 + cos)
                } as u8;
                let b = {
                    let dist = (xc * xc + yc * yc).sqrt();
                    let rate = SPEED * *nframes as f64;
                    let sin = (PI2 * (dist - rate) / PERIOD).sin();
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
    canvas.copy(streaming_texture, None, None)?;
    *nframes += 1;
    Ok(())
}

fn draw_graph(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    main_texture: &mut sdl2::render::Texture,
    streaming_texture: &mut sdl2::render::Texture,
    nframes: &mut usize,
) -> Result<(), String> {

    

    Ok(())
}
