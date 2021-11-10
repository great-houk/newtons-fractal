// All Used Windows
pub use self::graphing_window::GraphingWindow;

mod basic_window {
    use sdl2::render::Canvas;
    use sdl2::video::{Window, WindowPos};

    pub struct BasicWindowBuilder<'a> {
        video_subsystem: &'a sdl2::VideoSubsystem,
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
    }

    #[allow(dead_code)]
    impl BasicWindowBuilder<'_> {
        pub fn new<'a>(
            video_subsystem: &'a sdl2::VideoSubsystem,
            title: &'static str,
            width: u32,
            height: u32,
        ) -> BasicWindowBuilder<'a> {
            BasicWindowBuilder {
                video_subsystem,
                title,
                width,
                min_width: None,
                max_width: None,
                height,
                min_height: None,
                max_height: None,
                posx: WindowPos::Centered,
                posy: WindowPos::Centered,
                resizable: false,
                hidden: false,
                borderless: false,
                fullscreen: false,
            }
        }

        pub fn set_min_size(&mut self, width: Option<u32>, height: Option<u32>) -> &mut Self {
            self.min_width = width;
            self.min_height = height;
            self
        }

        pub fn set_max_size(&mut self, width: Option<u32>, height: Option<u32>) -> &mut Self {
            self.max_width = width;
            self.max_height = height;
            self
        }

        pub fn set_position(&mut self, posx: WindowPos, posy: WindowPos) -> &mut Self {
            self.posx = posx;
            self.posy = posy;
            self
        }

        pub fn set_resizable(&mut self, b: bool) -> &mut Self {
            self.resizable = b;
            self
        }
        pub fn set_hidden(&mut self, b: bool) -> &mut Self {
            self.hidden = b;
            self
        }
        pub fn set_borderless(&mut self, b: bool) -> &mut Self {
            self.borderless = b;
            self
        }
        pub fn set_fullscreen(&mut self, b: bool) -> &mut Self {
            self.fullscreen = b;
            self
        }
        pub fn build(&self) -> Result<BasicWindow, String> {
            BasicWindow::init(self)
        }
    }

    pub struct BasicWindow {
        pub canvas: Canvas<Window>,
    }
    impl BasicWindow {
        fn init(b: &BasicWindowBuilder) -> Result<BasicWindow, String> {
            let mut window = {
                let mut win = b.video_subsystem.window(b.title, b.width, b.height);
                win.position(
                    BasicWindow::to_ll_windowpos(b.posx),
                    BasicWindow::to_ll_windowpos(b.posy),
                );
                if b.resizable {
                    win.resizable();
                }
                if b.hidden {
                    win.hidden();
                }
                if b.borderless {
                    win.borderless();
                }
                if b.fullscreen {
                    win.fullscreen();
                }
                win.build().map_err(|e| e.to_string())?
            };
            match (b.min_width, b.min_height) {
                (Some(wid), Some(hei)) => window.set_minimum_size(wid, hei),
                (None, Some(hei)) => window.set_minimum_size(0, hei),
                (Some(wid), None) => window.set_minimum_size(wid, 0),
                (None, None) => Ok(()),
            }
            .map_err(|e| e.to_string())?;
            match (b.max_width, b.max_height) {
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
            Ok(BasicWindow { canvas })
        }
        pub fn window(&self) -> &Window {
            self.canvas.window()
        }
        pub fn window_mut(&mut self) -> &mut Window {
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
    use super::basic_window::{BasicWindow, BasicWindowBuilder};
    use sdl2::pixels::PixelFormatEnum;
    use sdl2::render::Texture;
    use sdl2::video::{Window, WindowPos};

    pub struct GraphingWindow {
        raw: BasicWindow,
        main_texture: Texture,
        graphing_texture: Texture,
    }

    impl GraphingWindow {
        pub fn init(
            video_subsystem: &sdl2::VideoSubsystem,
            title: &'static str,
            width: u32,
            height: u32,
            posx: WindowPos,
            posy: WindowPos,
            // iconPath: Option<>
        ) -> Result<GraphingWindow, String> {
            let window = BasicWindowBuilder::new(video_subsystem, title, width, height)
                .set_min_size(Some(500), Some(500))
                .set_position(posx, posy)
                .set_resizable(true)
                .build()?;

            let (main_texture, graphing_texture) = {
                let canv = &window.canvas;
                let m = canv
                    .create_texture_target(None, width, height)
                    .map_err(|e| e.to_string())?;
                let g = canv
                    .create_texture_streaming(PixelFormatEnum::ABGR8888, width, height)
                    .map_err(|e| e.to_string())?;
                (m, g)
            };

            Ok(GraphingWindow {
                raw: window,
                main_texture,
                graphing_texture,
            })
        }

        pub fn resized(&mut self, width: u32, height: u32) -> Result<(), String> {
            let size = (width as f32 * height as f32).sqrt() as u32;

            self.window_mut()
                .set_size(size, size)
                .map_err(|e| e.to_string())?;

            let (main_texture, graphing_texture) = {
                let canv = &self.raw.canvas;
                let m = canv
                    .create_texture_target(None, size, size)
                    .map_err(|e| e.to_string())?;
                let g = canv
                    .create_texture_streaming(PixelFormatEnum::ABGR8888, size, size)
                    .map_err(|e| e.to_string())?;
                (m, g)
            };
            self.main_texture = main_texture;
            self.graphing_texture = graphing_texture;

            Ok(())
        }

        pub fn window(&self) -> &Window {
            self.raw.window()
        }
        pub fn window_mut(&mut self) -> &mut Window {
            self.raw.window_mut()
        }
    }
}
