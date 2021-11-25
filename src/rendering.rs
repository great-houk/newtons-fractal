// Holds all the drawing logic, like the graph rendering and the settings display
pub use render_backend::main_loop;

mod drawing {
    pub struct Data {
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

    /// Initializes the Data type
    pub fn get_draw_data(width: u32, height: u32) -> Data {
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
        let (x_ratio, x_offset, y_ratio, y_offset) = get_mandelbrot_vals(
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

    /// Draws a single pixel, given an x in pixels, y in pixels, frame count, and Data
    pub fn draw_pixel(
        Data {
            x_ratio,
            x_offset,
            y_ratio,
            y_offset,
            ..
        }: &Data,
        _nframes: usize,
        pixel_x: u32,
        pixel_y: u32,
    ) -> (u8, u8, u8, u8) {
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

    /// Modifies the Data which is given to every pixel every frame.
    /// Only is called once per frame, after everything has been rendered
    pub fn modify_data(
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
        }: &mut Data,
        _nframes: usize,
    ) {
        let scale = 1.02;
        *window_width /= scale;
        *window_height /= scale;
        // Calculate useful numbers from those
        let (xr, xo, yr, yo) = get_mandelbrot_vals(
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

mod render_backend {
    use super::drawing::{draw_pixel, get_draw_data, modify_data, Data};
    use crate::windows::{Message, SafeTexture, ThreadMessage};
    use sdl2::rect::Rect;
    use std::alloc::{alloc, dealloc, Layout, LayoutError};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, RwLock,
    };
    use std::thread::{spawn, JoinHandle};

    pub fn main_loop(
        (mut texture_lock, sender, receiver): (
            Arc<SafeTexture>,
            Sender<ThreadMessage>,
            Receiver<Message>,
        ),
        monitor: Arc<AtomicBool>,
        mut width: u32,
        mut height: u32,
    ) -> Result<(), String> {
        // Initialize all variables
        let mut nframes = 0;
        let cores = num_cpus::get() * 2;
        let mut draw_data = Arc::new(RwLock::new(get_draw_data(width, height)));
        let (mut pixels, mut pixel_slices, mut splits) = init_pixels(width, height, cores)?;
        let (mut handles, mut senders, mut receivers) =
            start_threads(cores, splits, pixel_slices, draw_data.clone(), width)?;
        // Start main loop
        'main: while monitor.load(Ordering::Relaxed) {
            // Wait for the window to give up control on the textures
            match receiver.recv() {
                Ok(Message::DoneRender) => {
                    // Wait for threads to finish
                    for receiver in &receivers {
                        receiver.recv().unwrap();
                    }
                    // Grab the texture's lock again, and use them
                    {
                        let mut texture = match texture_lock.lock() {
                            Ok(t) => t,
                            Err(_) => return Err("The main mutex was poisoned!".to_string()),
                        };
                        // Copy buffer to the graphing texture
                        let slice = pixels.as_slice();
                        texture
                            .update(Rect::new(0, 0, width, height), slice, (width * 4) as usize)
                            .map_err(|e| e.to_string())?;
                    }
                    // Let main thread know we're done
                    sender
                        .send(ThreadMessage::RenderReady)
                        .expect("The Main Thread's Receiver is Dead!");
                    // Allow drawing logic to modify data
                    {
                        let mut mut_data = draw_data.try_write().unwrap();
                        modify_data(&mut mut_data, nframes);
                    }
                    // Restart rendering threads
                    for sender in &senders {
                        sender.send((true, nframes)).unwrap();
                    }
                }
                Ok(Message::Resize {
                    texture,
                    width: wid,
                    height: hei,
                }) => {
                    texture_lock = texture;
                    width = wid;
                    height = hei;
                    end_threads(senders, handles)?;
                    draw_data = Arc::new(RwLock::new(get_draw_data(width, height)));
                    let temp = init_pixels(width, height, cores)?;
                    pixels = temp.0;
                    pixel_slices = temp.1;
                    splits = temp.2;
                    let temp =
                        start_threads(cores, splits, pixel_slices, draw_data.clone(), width)?;
                    handles = temp.0;
                    senders = temp.1;
                    receivers = temp.2;
                }
                Ok(Message::Quit) => {
                    break 'main;
                }
                Err(_) => {
                    return Err(
                        "Something happened with the transmitter in main window!".to_string()
                    )
                }
            };
            nframes += 1;
        }
        // We are stopping, so all sub threads need to stop too
        end_threads(senders, handles)
    }

    type Thread = (
        Vec<JoinHandle<()>>,
        Vec<Sender<(bool, usize)>>,
        Vec<Receiver<bool>>,
    );
    fn start_threads(
        cores: usize,
        mut splits: Vec<usize>,
        mut pixel_slices: Vec<&'static mut [u8]>,
        data: Arc<RwLock<Data>>,
        width: u32,
    ) -> Result<Thread, String> {
        let mut handles = Vec::with_capacity(cores);
        let mut receivers = Vec::with_capacity(cores);
        let mut senders = Vec::with_capacity(cores);
        for _i in 0..cores {
            let slice = pixel_slices.pop().unwrap();
            let ind = splits.pop().unwrap();
            // let ind = splits[_i]; // Cool bug
            let thread_data = data.clone();
            let pitch = width as usize * 4;
            let (ttx, rx) = channel();
            let (tx, trx) = channel();
            tx.send((true, 0)).unwrap();
            let handle = spawn(move || {
                draw_loop(ttx, trx, slice, thread_data, ind, pitch);
            });
            receivers.push(rx);
            senders.push(tx);
            handles.push(handle);
        }
        Ok((handles, senders, receivers))
    }

    fn end_threads(
        senders: Vec<Sender<(bool, usize)>>,
        handles: Vec<JoinHandle<()>>,
    ) -> Result<(), String> {
        for sender in senders {
            sender.send((false, 0)).map_err(|e| e.to_string())?;
        }
        for handle in handles {
            handle.join().expect("Child Thread Panicked");
        }
        Ok(())
    }

    fn draw_loop(
        sender: Sender<bool>,
        receiver: Receiver<(bool, usize)>,
        slice: &mut [u8],
        data: Arc<RwLock<Data>>,
        ind: usize,
        pitch: usize,
    ) {
        while let Ok((true, nframes)) = receiver.recv() {
            // Modify pixels
            {
                let d = data.try_read().unwrap();
                draw_func(slice, &d, nframes, ind, pitch);
            }
            // Tell logic thread we're done
            sender.send(true).unwrap();
        }
    }

    fn draw_func(slice: &mut [u8], data: &Data, nframes: usize, ind: usize, pitch: usize) {
        for i in 0..slice.len() / 4 {
            let total_ind = i * 4 + ind;
            let (pixel_x, pixel_y) = {
                let x = (total_ind % pitch) / 4;
                let y = total_ind / pitch;
                (x as u32, y as u32)
            };
            let (r, g, b, a) = draw_pixel(data, nframes, pixel_x, pixel_y);
            slice[i * 4] = r;
            slice[i * 4 + 1] = g;
            slice[i * 4 + 2] = b;
            slice[i * 4 + 3] = a;
        }
    }

    type PixelSplits = (Pixels, Vec<&'static mut [u8]>, Vec<usize>);
    fn init_pixels(width: u32, height: u32, cores: usize) -> Result<PixelSplits, String> {
        let mut pixels = Pixels::new((width * height * 4) as usize).map_err(|e| e.to_string())?;
        let splits = {
            let dist = ((width * height) as usize / cores) * 4;
            let mut vec = Vec::with_capacity(cores as usize);
            for i in 0..cores {
                vec.push(i * dist);
            }
            vec
        };
        let slices = match unsafe { pixels.split_at_mut_unsafe(&splits[1..]) } {
            Some(s) => s,
            None => return Err("Failed to make pixel slices".to_string()),
        };
        Ok((pixels, slices, splits))
    }

    pub struct Pixels {
        ptr: *mut u8,
        len: usize,
    }
    #[allow(dead_code)]
    impl Pixels {
        /// Makes a new array of u8 size len. Panics if len == 0
        pub fn new(len: usize) -> Result<Self, LayoutError> {
            assert_eq!(len > 0, true);
            let ptr = unsafe {
                let layout = Layout::array::<u8>(len)?;
                alloc(layout) as *mut u8
            };
            Ok(Self { ptr, len })
        }
        pub fn as_slice(&self) -> &[u8] {
            unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
        }
        pub fn as_slice_mut(&mut self) -> &mut [u8] {
            unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
        }
        // Splits the array into slices based on the indices given in splits
        // If splits == [1, 5, 7] you will get a vec with slices len
        // 1, 4, 3.
        // It doesn't matter if you use the whole array. As long as all values of
        // splits are smaller than self.len() this will return Some()
        pub fn split_at(&self, splits: &[usize]) -> Option<Vec<Option<&[u8]>>> {
            let mut pointers = Vec::with_capacity(splits.len());
            let mut start = 0;
            for &split in splits {
                if split > self.len || split < start {
                    return None;
                }
                let pointer = unsafe {
                    let addr = self.ptr.add(start);
                    let len = split - start;
                    std::slice::from_raw_parts(addr, len)
                };
                pointers.push(Some(pointer));
                start = split;
            }
            if start < self.len {
                let pointer = unsafe {
                    let addr = self.ptr.add(start);
                    let len = self.len - start;
                    std::slice::from_raw_parts(addr, len)
                };
                pointers.push(Some(pointer));
            }
            Some(pointers)
        }
        /// Splits the array into slices based on the indices given in splits
        /// If splits == [1, 5, 7] you will get a vec with slices
        /// [0..1], [1..5], [5..7], [7..] if there is data left.
        /// As long as all values of
        /// splits are smaller than self.len() this will return Some()
        pub fn split_at_mut(&mut self, splits: &[usize]) -> Option<Vec<Option<&mut [u8]>>> {
            let mut pointers = Vec::with_capacity(splits.len());
            let mut start = 0;
            for &split in splits {
                if split > self.len || split < start {
                    return None;
                }
                let pointer = unsafe {
                    let addr = self.ptr.add(start);
                    let len = split - start;
                    std::slice::from_raw_parts_mut(addr, len)
                };
                pointers.push(Some(pointer));
                start = split;
            }
            if start < self.len {
                let pointer = unsafe {
                    let addr = self.ptr.add(start);
                    let len = self.len - start;
                    std::slice::from_raw_parts_mut(addr, len)
                };
                pointers.push(Some(pointer));
            }
            Some(pointers)
        }
        /// Splits the array into slices based on the indices given in splits
        /// If splits == [1, 5, 7] you will get a vec with slices
        /// [0..1], [1..5], [5..7], [7..] if there is data left.
        /// As long as all values of
        /// splits are smaller than self.len() this will return Some()
        /// This function is unsafe because the slices are not linked
        /// to the lifetime of this struct in any way. That means
        /// that you need to guarantee that the slices will be dropped
        /// first, or undefined behavior will happen.
        pub unsafe fn split_at_mut_unsafe<'a>(
            &mut self,
            splits: &[usize],
        ) -> Option<Vec<&'a mut [u8]>> {
            let mut slices = Vec::with_capacity(splits.len());
            let mut start = 0;
            for &split in splits {
                if split > self.len || split < start {
                    return None;
                }
                let slice = {
                    let addr = self.ptr.add(start);
                    let len = split - start;
                    std::slice::from_raw_parts_mut(addr, len)
                };
                slices.push(slice);
                start = split;
            }
            if start < self.len {
                let slice = {
                    let addr = self.ptr.add(start);
                    let len = self.len - start;
                    std::slice::from_raw_parts_mut(addr, len)
                };
                slices.push(slice);
            }
            Some(slices)
        }
        pub fn len(&self) -> usize {
            self.len
        }
    }

    impl Drop for Pixels {
        fn drop(&mut self) {
            unsafe {
                dealloc(
                    self.ptr as *mut u8,
                    Layout::array::<u8>(self.len).expect("Unreachable"),
                )
            };
        }
    }
}
