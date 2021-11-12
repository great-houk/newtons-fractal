// All Used Windows
pub use self::graphing_window::GraphingWindow;
pub use self::safe_texture::SafeTexture;

mod safe_texture {
    use sdl2::render::Texture;
    use std::ops::{Deref, DerefMut};
    use std::sync::Mutex;

    #[repr(transparent)]
    pub struct SafeTexture(Mutex<Texture>);

    impl SafeTexture {
        pub fn new(texture: Texture) -> Self {
            SafeTexture(Mutex::new(texture))
        }
    }

    impl Deref for SafeTexture {
        type Target = Mutex<Texture>;
        fn deref(&self) -> &Mutex<Texture> {
            &self.0
        }
    }

    impl DerefMut for SafeTexture {
        fn deref_mut(&mut self) -> &mut Mutex<Texture> {
            &mut self.0
        }
    }

    unsafe impl Send for SafeTexture {}
    unsafe impl Sync for SafeTexture {}
}

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
        pub fn present(&mut self) {
            self.canvas.present()
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
    use super::safe_texture::SafeTexture;
    use sdl2::pixels::PixelFormatEnum;
    use sdl2::render::BlendMode;
    use sdl2::video::{Window, WindowPos};
    use std::sync::{mpsc, Arc};

    pub struct GraphingWindow {
        pub raw: BasicWindow,
        main_texture: Arc<SafeTexture>,
        graphing_texture: Arc<SafeTexture>,
        signaler: Option<mpsc::Sender<bool>>,
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
                (Arc::new(SafeTexture::new(m)), Arc::new(SafeTexture::new(g)))
            };

            Ok(GraphingWindow {
                raw: window,
                main_texture,
                graphing_texture,
                signaler: None,
            })
        }

        pub fn resized(&mut self, width: u32, height: u32) -> Result<(), String> {
            let size = (width as f32 * height as f32).sqrt() as u32;

            self.window_mut()
                .set_size(size, size)
                .map_err(|e| e.to_string())?;

            self.send(false);

            Ok(())
        }

        pub fn remake_textures(&mut self) -> Result<(), String> {
            let (main_texture, graphing_texture) = {
                let canv = &self.raw.canvas;
                let (width, height) = self.raw.canvas.output_size()?;
                let m = canv
                    .create_texture_target(None, width, height)
                    .map_err(|e| e.to_string())?;
                let g = canv
                    .create_texture_streaming(PixelFormatEnum::ABGR8888, width, height)
                    .map_err(|e| e.to_string())?;
                (Arc::new(SafeTexture::new(m)), Arc::new(SafeTexture::new(g)))
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
        pub fn present(&mut self) -> Result<bool, String> {
            if let (Ok(mut main_tex), Ok(graph_tex)) = (
                self.main_texture.try_lock(),
                self.graphing_texture.try_lock(),
            ) {
                // Copy main_tex to graph tex
                main_tex.set_blend_mode(BlendMode::Blend);
                self.raw
                    .canvas
                    .with_texture_canvas(&mut main_tex, |canv| {
                        canv.copy(&graph_tex, None, None).unwrap();
                    })
                    .map_err(|e| e.to_string())?;
                // Render things
                self.raw.canvas.copy(&main_tex, None, None)?;
                self.send(true);
                self.raw.present();
                return Ok(true);
            }
            Ok(false)
        }
        pub fn get_textures(
            &mut self,
        ) -> (Arc<SafeTexture>, Arc<SafeTexture>, mpsc::Receiver<bool>) {
            let (tx, rx) = mpsc::channel();
            self.signaler = Some(tx);
            self.send(true);
            (self.main_texture.clone(), self.graphing_texture.clone(), rx)
        }
        // This function doesn't really care if it succeeds or not,
        // since if it fails the thread will be remade in the proper state anyway
        #[allow(unused_must_use)]
        fn send(&self, b: bool) {
            match self.signaler.as_ref() {
                Some(sig) => sig.send(b),
                None => panic!("Unreachable code in windows.rs get_textures()"),
            };
        }
    }
}
