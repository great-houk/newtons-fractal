pub use mandelbrot::Mandelbrot;

mod mandelbrot {
    use crate::rendering::{Pixel, Pixels, RenderOp};
    use crate::{MAIN_HEIGHT, MAIN_WIDTH};
    use sdl2::event::Event;
    use sdl2::rect::Rect;
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
        rect: Rect,
        pixels: Pixels,
        data: Data,
    }

    impl Mandelbrot {
        pub fn init() -> Box<Self> {
            let rect = Rect::new(0, 0, MAIN_WIDTH as u32, MAIN_HEIGHT as u32);
            let pixels = Pixels::new(MAIN_WIDTH, MAIN_HEIGHT).unwrap();
            let data = Self::init_data(MAIN_WIDTH as u32, MAIN_HEIGHT as u32);
            Box::new(Mandelbrot { rect, pixels, data })
        }

        fn init_data(width: u32, height: u32) -> Data {
            // Basic numbers
            let window_width = 3.;
            let window_height = 3.;
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

    impl RenderOp for Mandelbrot {
        fn get_rect(&self) -> &Rect {
            &self.rect
        }
        fn get_pixels(&self) -> &Pixels {
            &self.pixels
        }
        fn get_pixels_mut(&mut self) -> &mut Pixels {
            &mut self.pixels
        }
        fn draw_pixel(&self, pixel_x: usize, pixel_y: usize) -> Pixel {
            let Data {
                x_ratio,
                x_offset,
                y_ratio,
                y_offset,
                ..
            } = self.data;
            // (((2 * x) / width) - 1) * (windwid / 2) =  (windwid / width) * x - windwid / 2 - xoff;
            let x0 = x_ratio * pixel_x as f64 - x_offset;
            let y0 = y_ratio * pixel_y as f64 - y_offset;
            let mut x = 0.;
            let mut y = 0.;
            let mut iteration = 0;
            let it_mod;
            const MAX_ITERATION: usize = 1 << 10;
            // Here N = 2^8 is chosen as a reasonable bailout radius.

            while x * x + y * y <= (1 << 16) as f64 && iteration < MAX_ITERATION {
                let xtemp = x * x - y * y + x0;
                y = 2. * x * y + y0;
                x = xtemp;
                iteration = iteration + 1
            }
            // Used to avoid floating point issues with points inside the set.
            if iteration < MAX_ITERATION {
                // sqrt of inner term removed using log simplification rules.
                let log_zn = (x * x + y * y).ln() / 2.;
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
                const PALETTE: [(u8, u8, u8); 16] = [
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
        fn handle_event(&mut self, event: &Event) -> bool {
            match event {
                
            }
            false
        }
    }
}
