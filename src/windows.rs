// All Used Windows
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, RenderTarget, Texture};
use sdl2::video::{Window as SDL2Window, WindowPos};

pub struct WindowBuilder<'a> {
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
impl WindowBuilder<'_> {
    pub fn new<'a>(
        video_subsystem: &'a sdl2::VideoSubsystem,
        title: &'static str,
        width: u32,
        height: u32,
    ) -> WindowBuilder<'a> {
        WindowBuilder {
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
    pub fn build(&self) -> Result<Window, String> {
        Window::init(self)
    }
}

pub struct Window {
    canvas: Canvas<SDL2Window>,
    texture: Texture,
    width: usize,
    height: usize,
}
impl Window {
    fn init(builder: &WindowBuilder) -> Result<Window, String> {
        let mut window = {
            let mut win =
                builder
                    .video_subsystem
                    .window(builder.title, builder.width, builder.height);
            win.position(
                Window::to_ll_windowpos(builder.posx),
                Window::to_ll_windowpos(builder.posy),
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
        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;
        let texture =
            Self::make_texture(&mut canvas, builder.width as usize, builder.height as usize)?;
        Ok(Window {
            canvas,
            width: builder.width as usize,
            height: builder.height as usize,
            texture,
        })
    }
    pub fn canvas(&self) -> &Canvas<SDL2Window> {
        &self.canvas
    }
    pub fn canvas_mut(&mut self) -> &mut Canvas<SDL2Window> {
        &mut self.canvas
    }
    fn to_ll_windowpos(pos: WindowPos) -> i32 {
        match pos {
            WindowPos::Undefined => sdl2_sys::SDL_WINDOWPOS_UNDEFINED_MASK as i32,
            WindowPos::Centered => sdl2_sys::SDL_WINDOWPOS_CENTERED_MASK as i32,
            WindowPos::Positioned(x) => x as i32,
        }
    }
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    pub fn resized(&mut self, width: usize, height: usize) -> Result<(), String> {
        self.width = width;
        self.height = height;
        // Remake textures to fit
        self.texture = Self::make_texture(self.canvas_mut(), width, height)?;
        Ok(())
    }
    fn make_texture(
        canvas: &mut Canvas<impl RenderTarget>,
        width: usize,
        height: usize,
    ) -> Result<Texture, String> {
        let texture = canvas
            .create_texture_target(PixelFormatEnum::ABGR8888, width as u32, height as u32)
            .map_err(|e| e.to_string())?;
        Ok(texture)
    }
}
