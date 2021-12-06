// All Used Windows
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, Texture};
use sdl2::video::{Window as SDL2Window, WindowPos};

pub trait ResizeFn: Fn(usize, usize) -> (usize, usize) {}
impl<T: Fn(usize, usize) -> (usize, usize)> ResizeFn for T {}

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
    resize_func: Box<dyn ResizeFn>,
}

#[allow(dead_code)]
impl WindowBuilder<'_> {
    pub fn new<'a, T: 'static + ResizeFn>(
        video_subsystem: &'a sdl2::VideoSubsystem,
        title: &'static str,
        width: u32,
        height: u32,
        resize_func: T,
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
            resize_func: Box::new(resize_func),
        }
    }

    pub fn set_min_size(mut self, width: Option<u32>, height: Option<u32>) -> Self {
        self.min_width = width;
        self.min_height = height;
        self
    }

    pub fn set_max_size(mut self, width: Option<u32>, height: Option<u32>) -> Self {
        self.max_width = width;
        self.max_height = height;
        self
    }

    pub fn set_position(mut self, posx: WindowPos, posy: WindowPos) -> Self {
        self.posx = posx;
        self.posy = posy;
        self
    }

    pub fn set_resizable(mut self, b: bool) -> Self {
        self.resizable = b;
        self
    }
    pub fn set_hidden(mut self, b: bool) -> Self {
        self.hidden = b;
        self
    }
    pub fn set_borderless(mut self, b: bool) -> Self {
        self.borderless = b;
        self
    }
    pub fn set_fullscreen(mut self, b: bool) -> Self {
        self.fullscreen = b;
        self
    }
    pub fn build(self) -> Result<Window, String> {
        Window::init(self)
    }
}

pub struct Window {
    canvas: Canvas<SDL2Window>,
    texture: Texture,
    width: usize,
    height: usize,
    resize_func: Box<dyn ResizeFn>,
}
#[allow(dead_code)]
impl Window {
    fn init(builder: WindowBuilder) -> Result<Window, String> {
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
            resize_func: builder.resize_func,
        })
    }
    pub fn canvas(&self) -> &Canvas<SDL2Window> {
        &self.canvas
    }
    pub fn canvas_mut(&mut self) -> &mut Canvas<SDL2Window> {
        &mut self.canvas
    }
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    pub fn resized(&mut self, width: usize, height: usize) -> Result<(), String> {
        let (w, h) = (self.resize_func)(width, height);
        self.width = w;
        self.height = h;
        // Set window size if it needs to be changed
        if w != width || h != height {
            self.canvas_mut()
                .window_mut()
                .set_size(w as u32, h as u32)
                .map_err(|e| e.to_string())?;
        }
        // Remake textures to fit
        self.texture = Self::make_texture(self.canvas_mut(), w, h)?;
        Ok(())
    }
    pub fn present<'a, T: Into<&'a [u8]>>(&mut self, pixels: T, rect: Rect) {
        let data = pixels.into();
        let _b = &data;
        self.texture
            .update(rect, data, rect.width() as usize * 4)
            .unwrap();
        let texture = unsafe { &*(&self.texture as *const Texture) };
        self.canvas_mut().copy(texture, rect, rect).unwrap();
        self.canvas_mut().present();
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
    fn to_ll_windowpos(pos: WindowPos) -> i32 {
        match pos {
            WindowPos::Undefined => sdl2_sys::SDL_WINDOWPOS_UNDEFINED_MASK as i32,
            WindowPos::Centered => sdl2_sys::SDL_WINDOWPOS_CENTERED_MASK as i32,
            WindowPos::Positioned(x) => x as i32,
        }
    }
}

unsafe impl Send for Window {}
