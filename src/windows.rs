// All Used Windows

use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};

struct SDL2Window {
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

        let canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        let texture_creator = canvas.texture_creator();

        Ok(SDL2Window {
            canvas,
            texture_creator,
        })
    }

    fn window(&mut self) -> &mut Window {
        self.canvas.window_mut()
    }
}

pub mod graphing_window {
    use super::SDL2Window;
    use sdl2::video::WindowPos;

    pub struct Window {
        raw: SDL2Window,
    }

    pub fn init(
        video_subsystem: sdl2::VideoSubsystem,
        title: &'static str,
        width: u32,
        height: u32,
        posx: WindowPos,
        posy: WindowPos,
        // iconPath: Option<>
    ) -> Result<Window, String> {
        let mut window = SDL2Window::init(video_subsystem, title, width, height)?;
        {
            let win = window.window();
            win.set_position(posx, posy);
            win.hide();
        }

        Ok(Window { raw: window })
    }
}
