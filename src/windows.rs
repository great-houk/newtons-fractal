// All Used Windows

use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};

struct SDL2Window {
    window: Window,
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
}

impl SDL2Window {
    fn init(
        video_subsystem: sdl2::VideoSubsystem,
        title: &'static str,
        width: u32,
        height: u32,
    ) -> Result<SDL2Window, String> {
        let window = video_subsystem
            .window(title, width, height)
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        let texture_creator = canvas.texture_creator();

        Ok(SDL2Window {
            window,
            canvas,
            texture_creator,
        })
    }
}

pub mod GraphingWindow {
    use super::SDL2Window;

    pub struct Window {
        raw: SDL2Window,
    }

    pub fn init<'a>(
        video_subsystem: sdl2::VideoSubsystem,
        title: &'static str,
        width: u32,
        height: u32,
    ) -> Result<&'a Window, String> {
        let window = SDL2Window::init(video_subsystem, title, width, height)?;

        Ok(&Window { raw: window })
    }
}
