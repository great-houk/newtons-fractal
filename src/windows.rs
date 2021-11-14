// All Used Windows
pub use self::graphing_window::GraphingWindow;
pub use self::safe_texture::{Message, SafeTexture};

mod safe_texture {
    use sdl2::render::Texture;
    use std::ops::{Deref, DerefMut};
    use std::sync::{Arc, Mutex};

    pub enum Message {
        Resize {
            texture: Arc<SafeTexture>,
            width: u32,
            height: u32,
        },
        DoneRender,
        Quit,
    }

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
        fn init(builder: &BasicWindowBuilder) -> Result<BasicWindow, String> {
            let mut window = {
                let mut win =
                    builder
                        .video_subsystem
                        .window(builder.title, builder.width, builder.height);
                win.position(
                    BasicWindow::to_ll_windowpos(builder.posx),
                    BasicWindow::to_ll_windowpos(builder.posy),
                );
                if builder.resizable {
                    win.resizable();
                }
                if builder.hidden {
                    win.hidden();
                }
                if builder.borderless {
                    win.borderless();
                }
                if builder.fullscreen {
                    win.fullscreen();
                }
                win.build().map_err(|e| e.to_string())?
            };
            match (builder.min_width, builder.min_height) {
                (Some(wid), Some(hei)) => window.set_minimum_size(wid, hei),
                (None, Some(hei)) => window.set_minimum_size(0, hei),
                (Some(wid), None) => window.set_minimum_size(wid, 0),
                (None, None) => Ok(()),
            }
            .map_err(|e| e.to_string())?;
            match (builder.max_width, builder.max_height) {
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
    use super::safe_texture::{Message, SafeTexture};
    use sdl2::video::{Window, WindowPos};
    use std::sync::{mpsc, Arc};

    pub struct GraphingWindow {
        pub raw: BasicWindow,
        texture: Arc<SafeTexture>,
        signaler: Option<mpsc::Sender<Message>>,
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

            let texture = {
                let canv = &window.canvas;
                let m = canv
                    .create_texture_target(None, width, height)
                    .map_err(|e| e.to_string())?;
                Arc::new(SafeTexture::new(m))
            };

            Ok(GraphingWindow {
                raw: window,
                texture,
                signaler: None,
            })
        }

        pub fn resized(&mut self, width: u32, height: u32) -> Result<(u32, u32), String> {
            let size = (width as f32 * height as f32).sqrt() as u32;

            self.window_mut()
                .set_size(size, size)
                .map_err(|e| e.to_string())?;

            self.remake_textures()?;
            let texture = self.texture.clone();
            self.send(Message::Resize {
                texture,
                width: size,
                height: size,
            });

            Ok((size, size))
        }

        pub fn remake_textures(&mut self) -> Result<(), String> {
            let texture = {
                let canv = &self.raw.canvas;
                let (width, height) = self.raw.canvas.output_size()?;
                let m = canv
                    .create_texture_target(None, width, height)
                    .map_err(|e| e.to_string())?;
                Arc::new(SafeTexture::new(m))
            };
            self.texture = texture;
            Ok(())
        }

        pub fn window(&self) -> &Window {
            self.raw.window()
        }
        pub fn window_mut(&mut self) -> &mut Window {
            self.raw.window_mut()
        }
        pub fn present(&mut self) -> Result<bool, String> {
            if let Ok(main_tex) = self.texture.try_lock() {
                // Render things
                self.raw.canvas.copy(&main_tex, None, None)?;
                self.raw.present();
                self.send(Message::DoneRender);
                return Ok(true);
            }
            Ok(false)
        }
        pub fn get_textures(&mut self) -> (Arc<SafeTexture>, mpsc::Receiver<Message>) {
            let (tx, rx) = mpsc::channel();
            self.signaler = Some(tx);
            self.send(Message::DoneRender);
            (self.texture.clone(), rx)
        }
        // This function doesn't really care if it succeeds or not,
        // since if it fails the thread will be remade in the proper state anyway
        #[allow(unused_must_use)]
        pub fn send(&self, b: Message) {
            match self.signaler.as_ref() {
                Some(sig) => sig.send(b),
                None => panic!("Unreachable code in windows.rs get_textures()"),
            };
        }
    }
}
