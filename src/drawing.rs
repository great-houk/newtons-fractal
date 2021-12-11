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

mod mandelbrot {
    use super::basic_render_op::{BasicOp, BasicOpImp};
    use crate::events::{MainEvent, SdlEvent};
    use crate::rendering::{Pixel, Pixels, RenderOpReference};
    use crate::windows::Window;
    use sdl2::rect::Rect;
    use std::sync::{Arc, Mutex, RwLock};
    struct Data {
        x_ratio: f64,
        x_offset: f64,
        y_ratio: f64,
        y_offset: f64,
        max_iter: usize,
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
            let (w, h) = window.lock().unwrap().size();
            let data = Self::init_data(w as u32, h as u32);
            let op = BasicOp::init(window, w, h, 0, 0);
            Arc::new(RwLock::new(Box::new(Mandelbrot { data, op })))
        }

        fn init_data(width: u32, height: u32) -> Data {
            // Basic numbers
            let window_height = 1.;
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
                window_x,
                width,
                window_height,
                window_y,
                height,
            );
            let max_iter = 1 << 8;
            Data {
                x_ratio,
                x_offset,
                y_ratio,
                y_offset,
                max_iter,
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
            window_x: f64,
            width: u32,
            window_height: f64,
            window_y: f64,
            height: u32,
        ) -> (f64, f64, f64, f64) {
            let x_ratio = window_width / width as f64;
            let x_offset = window_width / 2. - window_x;
            let y_ratio = window_height / height as f64;
            let y_offset = window_height / 2. - window_y;
            (x_ratio, x_offset, y_ratio, y_offset)
        }

        fn draw_iter(&self, pixel_x: usize, pixel_y: usize) -> (Pixel, bool) {
            let Data {
                x_ratio,
                x_offset,
                y_ratio,
                y_offset,
                max_iter,
                ..
            } = self.data;
            // (((2 * x) / width) - 1) * (wind_wid / 2) =  (wind_wid / width) * x - wind_wid / 2 - x_off;
            let x0 = x_ratio * pixel_x as f64 - x_offset;
            let y0 = y_ratio * pixel_y as f64 - y_offset;
            let mut x_coord = x0;
            let mut y_coord = y0;
            let mut iteration = 0;
            let it_mod;

            while x_coord * x_coord + y_coord * y_coord <= (1 << 4) as f64 && iteration < max_iter {
                // Compute New Point
                let x_temp = x_coord * x_coord - y_coord * y_coord + x0;
                y_coord = 2. * x_coord * y_coord + y0;
                x_coord = x_temp;
                iteration += 1;
            }
            // Used to avoid floating point issues with points inside the set.
            if iteration < max_iter {
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
                return ((0, 0, 50, 255), true);
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
                // static PALETTE: [(u8, u8, u8); 2] = [(255, 255, 255), (255, 255, 255)];
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
            ((r, g, b, 255), false)
        }

        fn draw_rect(
            &self,
            ind: usize,
            i: usize,
            pixels: &mut [Pixel],
            (x_pos, y_pos, width, height): (usize, usize, usize, usize),
            pitch: usize,
        ) -> (usize, bool, bool) {
            let mut outside = false;
            let mut inside = false;
            let mut i = i + y_pos * pitch + x_pos;
            let mut y = ind / pitch + 2;
            for x in 0..width {
                let (color, black) = self.draw_iter(x + x_pos, y + y_pos);
                inside |= black;
                outside |= !black;
                pixels[i + x] = color;
            }
            i += pitch;
            // Calc side lines
            for _ in 1..height {
                let (color, black) = self.draw_iter(x_pos, y + y_pos);
                inside |= black;
                outside |= !black;
                pixels[i] = color;
                let (color, black) = self.draw_iter(width - 1 + x_pos, y + y_pos);
                inside |= black;
                outside |= !black;
                pixels[i + width - 1] = color;
                i += pitch;
                y += 1;
            }
            // Calc bottom line
            i -= pitch - 1;
            y -= 1;
            for x in 1..width - 1 {
                let (color, black) = self.draw_iter(x + x_pos, y + y_pos);
                inside |= black;
                outside |= !black;
                pixels[i] = color;
                i += 1;
            }
            (i, inside, outside)
        }

        fn fill_rect(
            &self,
            ind: usize,
            i: usize,
            pixels: &mut [Pixel],
            (x_pos, y_pos, width, height): (usize, usize, usize, usize),
            pitch: usize,
            color: bool,
        ) {
            if color {
                let mut i = i + y_pos * pitch + x_pos;
                let mut y = ind / pitch + 2;
                for _ in 0..height - 1 {
                    for x in 1..width - 1 {
                        let (color, _) = self.draw_iter(x + x_pos, y + y_pos);
                        pixels[i + x] = color;
                    }
                    y += 1;
                    i += pitch;
                }
            } else {
                let mut i = i + y_pos * pitch + x_pos;
                for _ in 0..height - 1 {
                    for x in 1..width - 1 {
                        pixels[i + x] = (0, 0, 0, 0);
                    }
                    i += pitch;
                }
            }
        }

        fn rect_divide(
            &self,
            rects: &mut Vec<(usize, usize, usize, usize)>,
            ind: usize,
            i: usize,
            pixels: &mut [Pixel],
            pitch: usize,
        ) -> usize {
            let rect = rects.pop().unwrap();
            let (i_final, inside, outside) = self.draw_rect(ind, i, pixels, rect, pitch);
            // The border rendered had only inside points or outside points,
            // so fill the rect with either black or color
            if inside ^ outside {
                self.fill_rect(ind, i, pixels, rect, pitch, outside);
            }
            // Else break up the rectangle until it's good
            else {
                if rect.2 < 6 || rect.3 < 6 {
                    self.fill_rect(ind, i, pixels, rect, pitch, outside);
                    return i_final;
                }

                if rect.2 > rect.3 {
                    let x1 = rect.0 + 1;
                    let y1 = rect.1 + 1;
                    let w1 = rect.2 / 2;
                    let h1 = rect.3 - 2;
                    let x2 = rect.0 + rect.2 / 2 + 1;
                    let w2 = rect.2 - 2 - w1;

                    let rect1 = (x1, y1, w1, h1);
                    let rect3 = (x2, y1, w2, h1);

                    rects.push(rect1);
                    rects.push(rect3);
                } else {
                    let x1 = rect.0 + 1;
                    let y1 = rect.1 + 1;
                    let w1 = rect.2 - 2;
                    let h1 = rect.3 / 2;
                    let y2 = rect.1 + rect.3 / 2 + 1;
                    let h2 = rect.3 - 2 - h1;

                    let rect1 = (x1, y1, w1, h1);
                    let rect2 = (x1, y2, w1, h2);

                    rects.push(rect1);
                    rects.push(rect2);
                }
            }
            i_final
        }

        fn zoom(&mut self, factor: f64) {
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
                ..
            } = &mut self.data;
            //
            *window_width /= factor;
            *window_height /= factor;
            // Calculate useful numbers from those
            let (xr, xo, yr, yo) = Self::get_mandelbrot_vals(
                *window_width,
                *window_x,
                *width,
                *window_height,
                *window_y,
                *height,
            );
            *x_ratio = xr;
            *x_offset = xo;
            *y_ratio = yr;
            *y_offset = yo;
        }

        fn translate(&mut self, x_percent: f64, y_percent: f64) {
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
                ..
            } = &mut self.data;
            // Calc new window_x and window_y
            let x_dist = *window_width * x_percent;
            *window_x += x_dist;
            let y_dist = *window_height * y_percent;
            *window_y -= y_dist;
            // Calculate useful numbers from those
            let (xr, xo, yr, yo) = Self::get_mandelbrot_vals(
                *window_width,
                *window_x,
                *width,
                *window_height,
                *window_y,
                *height,
            );
            *x_ratio = xr;
            *x_offset = xo;
            *y_ratio = yr;
            *y_offset = yo;
        }
    }

    impl BasicOpImp for Mandelbrot {
        fn get_op(&self) -> &BasicOp {
            &self.op
        }
        fn get_op_mut(&mut self) -> &mut BasicOp {
            &mut self.op
        }
        fn draw(&self, pixels: &mut [Pixel], ind: usize, pitch: usize) {
            // Calc all top pixels
            let mut i = 0;
            while (i + ind) % pitch != 0 {
                let total_ind = i + ind;
                let (pixel_x, pixel_y) = Self::ind_to_xy(total_ind, pitch);
                let (color, _) = self.draw_iter(pixel_x, pixel_y);
                pixels[i] = color;
                i += 1;
            }
            // Calc middle rect
            let height = (pixels.len() - i) / pitch;
            let mut rects = vec![(0, 0, pitch, height)];
            // Start rect logic
            let i_final = self.rect_divide(&mut rects, ind, i, pixels, pitch);
            while !rects.is_empty() {
                self.rect_divide(&mut rects, ind, i, pixels, pitch);
            }
            // Calc bottom pixels
            for j in i_final..pixels.len() {
                let total_ind = j + ind;
                let (pixel_x, pixel_y) = Self::ind_to_xy(total_ind, pitch);
                let (color, _) = self.draw_iter(pixel_x, pixel_y);
                pixels[j] = color;
            }
        }
        fn modify_data(&mut self) {}
        fn handle_events(&mut self) -> bool {
            use sdl2::event::{Event, WindowEvent};

            let s = self.get_op_mut();
            let list = s.event_list.get_mut().unwrap().split_off(0);
            let mut ret = false;

            for event in list {
                // println!("{:?}", event);
                match event {
                    // We finished rendering, and we want to be constantly rendering,
                    // so send true to start again.
                    SdlEvent::User(MainEvent::RenderOpFinish(op)) => {
                        // If we can't read the op, then it must be this
                        // so we finished, and there's no reason to not
                        // restart, so send true back
                        if op.try_read().is_err() {
                            ret = true;
                        }
                    }
                    // Window was resized, so do a ton
                    SdlEvent::Event(Event::Window {
                        win_event: WindowEvent::Resized(wid, hei),
                        window_id: win_id,
                        ..
                    }) => {
                        if win_id != self.get_op().id() {
                            continue;
                        }

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
                            d.window_x,
                            d.width,
                            d.window_height,
                            d.window_y,
                            d.height,
                        );
                        d.x_ratio = xr;
                        d.x_offset = xo;
                        d.y_ratio = yr;
                        d.y_offset = yo;
                    }
                    // User did some keyboard input
                    SdlEvent::Event(Event::KeyDown {
                        window_id,
                        scancode,
                        ..
                    }) => {
                        if window_id != self.get_op().id() {
                            continue;
                        }

                        use sdl2::keyboard::Scancode;
                        const MOVE_AMOUNT: f64 = 0.1;
                        const SCALE_COARSE: f64 = 1.6;
                        match scancode.unwrap() {
                            Scancode::W => self.translate(0., MOVE_AMOUNT),
                            Scancode::A => self.translate(-MOVE_AMOUNT, 0.),
                            Scancode::S => self.translate(0., -MOVE_AMOUNT),
                            Scancode::D => self.translate(MOVE_AMOUNT, 0.),
                            Scancode::Q => self.zoom(SCALE_COARSE),
                            Scancode::E => self.zoom(2. - SCALE_COARSE),
                            Scancode::Up => {
                                self.data.max_iter <<= 1;
                                println!("Scale: {}", (self.data.max_iter as f64).log(2.))
                            }
                            Scancode::Down => self.data.max_iter >>= 1,
                            Scancode::Left => {}
                            Scancode::Right => {}
                            _ => {}
                        }
                    }
                    _ => (),
                }
            }
            ret
        }
    }
}
