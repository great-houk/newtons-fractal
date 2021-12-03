pub use mandelbrot::Mandelbrot;

mod basic_render_op {
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
        fn draw_pixel(&self, x: usize, y: usize) -> Pixel;
        fn modify_data(&mut self);
        fn handle_events(&mut self) -> bool;
    }

    pub struct BasicOp {
        pub window: Arc<Mutex<Window>>,
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
            let rect = Rect::new(x as i32, y as i32, width as u32, height as u32);
            let buffer1 = Pixels::new(width, height).unwrap();
            let buffer2 = Pixels::new(width, height).unwrap();
            let buffers = [buffer1, buffer2];
            let buffer_ind = 0;
            let event_list = Mutex::new(vec![]);
            BasicOp {
                window,
                rect,
                buffers,
                buffer_ind,
                event_list,
                open: AtomicBool::new(true),
            }
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
        fn draw_pixel(&self, x: usize, y: usize) -> Pixel {
            self.draw_pixel(x, y)
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

mod mandelbrot {
    use super::basic_render_op::{BasicOp, BasicOpImp};
    use crate::events::{MainEvent, SdlEvent};
    use crate::rendering::{Pixel, Pixels, RenderOpReference};
    use crate::windows::Window;
    use crate::{MAIN_HEIGHT, MAIN_WIDTH};
    use sdl2::rect::Rect;
    use std::sync::{Arc, Mutex, RwLock};
    struct Data {
        x_ratio: f64,
        x_offset: f64,
        y_ratio: f64,
        y_offset: f64,
        //
        window_width: f64,
        window_height: f64,
        window_x: f64,
        window_y: f64,
        width: u32,
        height: u32,
    }

    pub struct Mandelbrot {
        data: Data,
        op: BasicOp,
    }

    impl Mandelbrot {
        pub fn init(window: Arc<Mutex<Window>>) -> RenderOpReference {
            let data = Self::init_data(MAIN_WIDTH as u32, MAIN_HEIGHT as u32);
            let op = BasicOp::init(window, MAIN_WIDTH, MAIN_HEIGHT, 0, 0);
            Arc::new(RwLock::new(Box::new(Mandelbrot { data, op })))
        }

        fn init_data(width: u32, height: u32) -> Data {
            // Basic numbers
            let window_height = 5.;
            let window_width = window_height * (width as f64 / height as f64);
            // 0.001643721971153 − 0.822467633298876i
            // -0.761574 - 0.0847596i
            // -e/7 - e/20i
            // -0.10715079727776 - 0.91210278793461i
            // -1.74790375491685 + 0.00194820459426i
            // -0.52303294558693 + 0.52633610977926i
            //
            let window_x = 0.001643721971153;
            let window_y = -0.822467633298876;
            // Calculate useful numbers from those
            let (x_ratio, x_offset, y_ratio, y_offset) = Self::get_mandelbrot_vals(
                window_width,
                width,
                window_x,
                window_height,
                height,
                window_y,
            );
            Data {
                x_ratio,
                x_offset,
                y_ratio,
                y_offset,
                //
                window_width,
                window_height,
                window_x,
                window_y,
                width,
                height,
            }
        }

        fn get_mandelbrot_vals(
            window_width: f64,
            width: u32,
            window_x: f64,
            window_height: f64,
            height: u32,
            window_y: f64,
        ) -> (f64, f64, f64, f64) {
            let x_ratio = window_width / width as f64;
            let x_offset = window_width / 2. - window_x;
            let y_ratio = window_height / height as f64;
            let y_offset = window_height / 2. - window_y;
            (x_ratio, x_offset, y_ratio, y_offset)
        }

        
    }

    impl BasicOpImp for Mandelbrot {
        fn get_op(&self) -> &BasicOp {
            &self.op
        }
        fn get_op_mut(&mut self) -> &mut BasicOp {
            &mut self.op
        }
        fn draw_pixel(&self, pixel_x: usize, pixel_y: usize) -> Pixel {
            let Data {
                x_ratio,
                x_offset,
                y_ratio,
                y_offset,
                ..
            } = self.data;
            // (((2 * x) / width) - 1) * (wind_wid / 2) =  (wind_wid / width) * x - wind_wid / 2 - x_off;
            let x0 = x_ratio * pixel_x as f64 - x_offset;
            let y0 = y_ratio * pixel_y as f64 - y_offset;
            let mut x_coord = 0.;
            let mut y_coord = 0.;
            let mut iteration = 0;
            let it_mod;
            const MAX_ITERATION: usize = 1 << 10;
            // Here N = 2^8 is chosen as a reasonable bailout radius.

            while x_coord * x_coord + y_coord * y_coord <= (1 << 16) as f64
                && iteration < MAX_ITERATION
            {
                let x_temp = x_coord * x_coord - y_coord * y_coord + x0;
                y_coord = 2. * x_coord * y_coord + y0;
                x_coord = x_temp;
                iteration += 1;
            }
            // Used to avoid floating point issues with points inside the set.
            if iteration < MAX_ITERATION {
                // sqrt of inner term removed using log simplification rules.
                let log_zn = (x_coord * x_coord + y_coord * y_coord).ln() / 2.;
                let nu = (log_zn / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
                // Rearranging the potential function.
                // Dividing log_zn by log(2) instead of log(N = 1<<8)
                // because we want the entire palette to range from the
                // center to radius 2, NOT our bailout radius.
                let it = iteration as f64 + 1. - nu;
                iteration = it as usize;
                it_mod = it % 1.;
            } else {
                return (0, 0, 0, 255);
            }
            // Color choosing
            let (r, g, b) = {
                static PALETTE: [(u8, u8, u8); 16] = [
                    (66, 30, 15),
                    (25, 7, 26),
                    (9, 1, 47),
                    (4, 4, 73),
                    (0, 7, 100),
                    (12, 44, 138),
                    (24, 82, 177),
                    (57, 125, 209),
                    (134, 181, 229),
                    (211, 236, 248),
                    (241, 233, 191),
                    (248, 201, 95),
                    (255, 170, 0),
                    (204, 128, 0),
                    (153, 87, 0),
                    (106, 52, 3),
                ];
                let color1 = PALETTE[iteration % PALETTE.len()];
                let color2 = PALETTE[(iteration + 1) % PALETTE.len()];
                // let (dr, dg, db) = (0., 0., 0.);
                let (dr, dg, db) = (
                    color2.0 as f64 - color1.0 as f64,
                    color2.1 as f64 - color1.1 as f64,
                    color2.2 as f64 - color1.2 as f64,
                );
                (
                    (color1.0 as f64 + (dr * it_mod)) as u8,
                    (color1.1 as f64 + (dg * it_mod)) as u8,
                    (color1.2 as f64 + (db * it_mod)) as u8,
                )
            };
            (r, g, b, 255)
        }
        fn modify_data(&mut self) {
            let Data {
                ref mut x_ratio,
                ref mut x_offset,
                ref mut y_ratio,
                ref mut y_offset,
                //
                ref mut window_width,
                ref mut window_height,
                ref mut window_x,
                ref mut window_y,
                ref mut width,
                ref mut height,
            } = &mut self.data;
            //
            let scale = 1.02;
            *window_width /= scale;
            *window_height /= scale;
            // Calculate useful numbers from those
            let (xr, xo, yr, yo) = Self::get_mandelbrot_vals(
                *window_width,
                *width,
                *window_x,
                *window_height,
                *height,
                *window_y,
            );
            *x_ratio = xr;
            *x_offset = xo;
            *y_ratio = yr;
            *y_offset = yo;
        }
        fn handle_events(&mut self) -> bool {
            use sdl2::event::{Event, WindowEvent};

            let s = self.get_op_mut();
            let list = s.event_list.get_mut().unwrap().split_off(0);
            let mut ret = false;
            for event in list {
                // println!("{:?}", event);
                match event {
                    SdlEvent::User(MainEvent::RenderOpFinish(op)) => {
                        // If we can't read the op, then it must be this
                        // so we finished, and there's no reason to not
                        // restart, so send true back
                        if let Err(_) = op.try_read() {
                            ret = true;
                        }
                    }
                    SdlEvent::Event(Event::Window {
                        win_event: WindowEvent::Resized(wid, hei),
                        ..
                    }) => {
                        let s = self.get_op_mut();
                        s.rect = Rect::new(0, 0, wid as u32, hei as u32);
                        let buffer1 = Pixels::new(wid as usize, hei as usize).unwrap();
                        let buffer2 = Pixels::new(wid as usize, hei as usize).unwrap();
                        s.buffers = [buffer1, buffer2];
                        s.buffer_ind = 0;

                        let d = &mut self.data;
                        d.window_width *= wid as f64 / d.width as f64;
                        d.window_height *= hei as f64 / d.height as f64;
                        d.width = wid as u32;
                        d.height = hei as u32;

                        let (xr, xo, yr, yo) = Self::get_mandelbrot_vals(
                            d.window_width,
                            d.width,
                            d.window_x,
                            d.window_height,
                            d.height,
                            d.window_y,
                        );
                        d.x_ratio = xr;
                        d.x_offset = xo;
                        d.y_ratio = yr;
                        d.y_offset = yo;
                    }
                    _ => (),
                }
            }
            ret
        }
    }
}
