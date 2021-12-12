mod mandelbrot;
pub use mandelbrot::Mandelbrot;

pub mod basic_render_op {
    use crate::events::SdlEvent;
    use crate::rendering::{Pixel, Pixels, RenderOp};
    use crate::windows::Window;
    use sdl2::rect::Rect;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };

    pub trait BasicOpImp: Send + Sync {
        fn get_op(&self) -> &BasicOp;
        fn get_op_mut(&mut self) -> &mut BasicOp;
        fn draw(&self, pixels: &mut [Pixel], ind: usize, slice: usize);
        fn modify_data(&mut self);
        fn handle_events(&mut self) -> bool;
        fn ind_to_xy(ind: usize, pitch: usize) -> (usize, usize) {
            let x = ind % pitch;
            let y = ind / pitch;
            (x, y)
        }
    }

    pub struct BasicOp {
        pub window: Arc<Mutex<Window>>,
        pub window_id: u32,
        pub rect: Rect,
        pub buffers: [Pixels; 2],
        pub buffer_ind: usize,
        pub event_list: Mutex<Vec<SdlEvent>>,
        pub open: AtomicBool,
    }

    impl BasicOp {
        pub fn init(
            window: Arc<Mutex<Window>>,
            width: usize,
            height: usize,
            x: isize,
            y: isize,
        ) -> Self {
            let window_id = window.lock().unwrap().id();
            let rect = Rect::new(x as i32, y as i32, width as u32, height as u32);
            let buffer1 = Pixels::new(width, height).unwrap();
            let buffer2 = Pixels::new(width, height).unwrap();
            let buffers = [buffer1, buffer2];
            let buffer_ind = 0;
            let event_list = Mutex::new(vec![]);
            BasicOp {
                window,
                window_id,
                rect,
                buffers,
                buffer_ind,
                event_list,
                open: AtomicBool::new(true),
            }
        }
        pub fn id(&self) -> u32 {
            self.window_id
        }
    }

    impl<T: BasicOpImp> RenderOp for T {
        fn get_window(&self) -> Arc<Mutex<Window>> {
            self.get_op().window.clone()
        }
        fn get_rect(&self) -> &Rect {
            &self.get_op().rect
        }
        fn get_present_buffer(&self) -> &Pixels {
            let s = self.get_op();
            &s.buffers[(s.buffer_ind + 1) % 2]
        }
        fn get_draw_buffer(&self) -> &Pixels {
            let s = self.get_op();
            &s.buffers[s.buffer_ind]
        }
        fn swap_buffers(&mut self) {
            self.get_op_mut().buffer_ind += 1;
            self.get_op_mut().buffer_ind %= 2;
        }
        fn draw(&self, pixels: &mut [Pixel], ind: usize, pitch: usize) {
            self.draw(pixels, ind, pitch);
        }
        fn modify_data(&mut self) {
            self.modify_data();
        }
        fn handle_events(&mut self) -> bool {
            self.handle_events()
        }
        fn push_event(&self, event: SdlEvent) {
            let mut list = self.get_op().event_list.lock().unwrap();
            list.push(event);
        }
        fn set_open(&self, state: bool) {
            let s = self.get_op();
            s.open.store(state, Ordering::Relaxed);
        }
        fn get_open(&self) -> bool {
            let s = self.get_op();
            s.open.load(Ordering::Relaxed)
        }
    }
}
