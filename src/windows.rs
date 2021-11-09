// All Used Windows
pub use self::graphing_window::GraphingWindow;

mod basic_window {
    use sdl2::render::{Canvas, TextureCreator};
    use sdl2::video::{Window, WindowContext, WindowPos};

    pub struct BasicWindow {
        canvas: Canvas<Window>,
        texture_creator: TextureCreator<WindowContext>,
    }
    impl BasicWindow {
        pub fn init(
            video_subsystem: sdl2::VideoSubsystem,
            title: &'static str,
            width: u32,
            min_width: Option<u32>,
            max_width: Option<u32>,
            height: u32,
            min_height: Option<u32>,
            max_height: Option<u32>,
            posx: WindowPos,
            posy: WindowPos,
            resizable: bool,
            hidden: bool,
            borderless: bool,
            fullscreen: bool,
        ) -> Result<BasicWindow, String> {
            let mut window = {
                let mut win = video_subsystem.window(title, width, height);
                win.position(
                    BasicWindow::to_ll_windowpos(posx),
                    BasicWindow::to_ll_windowpos(posy),
                );
                if resizable {
                    win.resizable();
                }
                if hidden {
                    win.hidden();
                }
                if borderless {
                    win.borderless();
                }
                if fullscreen {
                    win.fullscreen();
                }
                win.build().map_err(|e| e.to_string())?
            };
            match (min_width, min_height) {
                (Some(wid), Some(hei)) => window.set_minimum_size(wid, hei),
                (None, Some(hei)) => window.set_minimum_size(0, hei),
                (Some(wid), None) => window.set_minimum_size(wid, 0),
                (None, None) => Ok(()),
            }
            .map_err(|e| e.to_string())?;
            match (max_width, max_height) {
                (Some(wid), Some(hei)) => window.set_maximum_size(wid, hei),
                (None, Some(hei)) => window.set_maximum_size(0, hei),
                (Some(wid), None) => window.set_maximum_size(wid, 0),
                (None, None) => Ok(()),
            }
            .map_err(|e| e.to_string())?;
            let canvas = window
                .into_canvas()
                .present_vsync()
                .build()
                .map_err(|e| e.to_string())?;
            let texture_creator = canvas.texture_creator();
            Ok(BasicWindow {
                canvas,
                texture_creator,
            })
        }
        pub fn window(&mut self) -> &mut Window {
            self.canvas.window_mut()
        }
        fn to_ll_windowpos(pos: WindowPos) -> i32 {
            match pos {
                WindowPos::Undefined => sdl2_sys::SDL_WINDOWPOS_UNDEFINED_MASK as i32,
                WindowPos::Centered => sdl2_sys::SDL_WINDOWPOS_CENTERED_MASK as i32,
                WindowPos::Positioned(x) => x as i32,
            }
        }
    }
}

mod graphing_window {
    use super::basic_window::BasicWindow;
    use sdl2::video::{Window, WindowPos};

    pub struct GraphingWindow {
        raw: BasicWindow,
    }

    impl GraphingWindow {
        pub fn init(
            video_subsystem: sdl2::VideoSubsystem,
            title: &'static str,
            width: u32,
            height: u32,
            posx: WindowPos,
            posy: WindowPos,
            // iconPath: Option<>
        ) -> Result<GraphingWindow, String> {
            let window = BasicWindow::init(
                video_subsystem,
                title,
                width,
                Some(500),
                None,
                height,
                Some(500),
                None,
                posx,
                posy,
                true,
                false,
                false,
                false,
            )?;

            Ok(GraphingWindow { raw: window })
        }

        pub fn resized(&mut self, width: u32, height: u32) -> Result<(), String> {
            let size = (width as f32 * height as f32).sqrt() as u32;

            self.window()
                .set_size(size, size)
                .map_err(|e| e.to_string())?;

            Ok(())
        }

        pub fn window(&mut self) -> &mut Window {
            self.raw.window()
        }
    }
}
