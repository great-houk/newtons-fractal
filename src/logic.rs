// Holds all the drawing logic, like the graph rendering and the settings display
// Also parses text and makes equation logic
pub use render_backend::main_loop;

mod render_backend {
    use super::drawing::{draw_pixel, get_draw_data, Data};
    use crate::windows::SafeTexture;
    use std::alloc::{alloc, dealloc, Layout, LayoutError};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    };
    use std::thread::{spawn, JoinHandle};
    pub fn main_loop(
        (texture_lock, sender): (Arc<SafeTexture>, Receiver<bool>),
        monitor: Arc<AtomicBool>,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        // Initialize all variables
        let cores = num_cpus::get() * 2;
        let data = get_draw_data(width);
        let (pixels, slices, splits) = init_pixels(width, height, cores)?;
        let (handles, senders, receivers) =
            init_threads(cores, splits, slices, data.clone(), width)?;
        // Start main loop
        'main: while monitor.load(Ordering::Relaxed) {
            // Wait for the window to give up control on the textures
            match sender.recv() {
                Ok(true) => (),
                Ok(false) => break 'main,
                Err(_) => {
                    return Err(
                        "Something happened with the transmitter in main window!".to_string()
                    )
                }
            };
            {
                // Grab the texture's lock again, and use them
                let mut texture = match texture_lock.lock() {
                    Ok(t) => t,
                    Err(_) => return Err("The main mutex was poisoned!".to_string()),
                };
                // Wait for threads to finish
                for receiver in &receivers {
                    receiver.recv().unwrap();
                }
                // Copy buffer to the graphing texture
                let slice = pixels.as_slice();
                texture
                    .update(None, slice, (width * 4) as usize)
                    .map_err(|e| e.to_string())?;
                // Restart rendering threads
                for sender in &senders {
                    sender.send(true).unwrap();
                }
            }
            // Once we finished using the textures, presumably
            // the main thread will update the screen and send us a message
        }
        // We are stopping, so all sub threads need to stop too
        for sender in senders {
            sender.send(false).unwrap();
        }
        for handle in handles {
            handle.join().expect("Child Thread Panicked");
        }
        Ok(())
    }

    fn init_pixels(
        width: u32,
        height: u32,
        cores: usize,
    ) -> Result<(Pixels, Vec<Option<&'static mut [u8]>>, Vec<usize>), String> {
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

    fn init_threads(
        cores: usize,
        splits: Vec<usize>,
        mut slices: Vec<Option<&'static mut [u8]>>,
        data: Data,
        width: u32,
    ) -> Result<(Vec<JoinHandle<()>>, Vec<Sender<bool>>, Vec<Receiver<bool>>), String> {
        let mut handles = Vec::with_capacity(cores);
        let mut receivers = Vec::with_capacity(cores);
        let mut senders = Vec::with_capacity(cores);
        for i in 0..cores {
            let slice = match slices[i].take() {
                Some(t) => t,
                None => return Err("Unreachable".to_string()),
            };
            let ind = splits[i];
            let pitch = width as usize * 4;
            let (ttx, rx) = channel();
            let (tx, trx) = channel();
            tx.send(true).unwrap();
            let handle = spawn(move || {
                draw_loop(ttx, trx, slice, &data, ind, pitch);
            });
            receivers.push(rx);
            senders.push(tx);
            handles.push(handle);
        }
        Ok((handles, senders, receivers))
    }

    fn draw_loop(
        sender: Sender<bool>,
        receiver: Receiver<bool>,
        slice: &mut [u8],
        data: &Data,
        ind: usize,
        pitch: usize,
    ) {
        let mut nframes = 0;
        while let Ok(true) = receiver.recv() {
            draw_func(slice, data, nframes, ind, pitch);
            sender.send(true).unwrap();
            nframes += 1;
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
    pub struct Pixels {
        ptr: *mut u8,
        len: usize,
    }
    #[allow(dead_code)]
    impl Pixels {
        pub fn new(len: usize) -> Result<Self, LayoutError> {
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
        // Splits the array into slices based on the indices given in splits
        // If splits == [1, 5, 7] you will get a vec with slices
        // [0..1], [1..5], [5..7], [7..] if there is data left.
        // As long as all values of
        // splits are smaller than self.len() this will return Some()
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
        // Splits the array into slices based on the indices given in splits
        // If splits == [1, 5, 7] you will get a vec with slices
        // [0..1], [1..5], [5..7], [7..] if there is data left.
        // As long as all values of
        // splits are smaller than self.len() this will return Some()
        // This function is unsafe because the slices are not linked
        // to the lifetime of this struct in any way. That means
        // that you need to guarantee that the slices will be dropped
        // first, or undefined behavior will happen.
        pub unsafe fn split_at_mut_unsafe<'a>(
            &mut self,
            splits: &[usize],
        ) -> Option<Vec<Option<&'a mut [u8]>>> {
            let mut pointers = Vec::with_capacity(splits.len());
            let mut start = 0;
            for &split in splits {
                if split > self.len || split < start {
                    return None;
                }
                let pointer = {
                    let addr = self.ptr.add(start);
                    let len = split - start;
                    std::slice::from_raw_parts_mut(addr, len)
                };
                pointers.push(Some(pointer));
                start = split;
            }
            if start < self.len {
                let pointer = {
                    let addr = self.ptr.add(start);
                    let len = self.len - start;
                    std::slice::from_raw_parts_mut(addr, len)
                };
                pointers.push(Some(pointer));
            }
            Some(pointers)
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

mod drawing {
    pub type Data = (f64, f64, f64, f64);

    pub fn get_draw_data(width: u32) -> Data {
        let center = width as f64 / 2.;
        let speed = width as f64 / 500.;
        let period = width as f64 / 10.;
        let pi2 = std::f64::consts::PI * 2.;
        (center, speed, period, pi2)
    }

    pub fn draw_pixel(
        (center, speed, period, pi2): &Data,
        nframes: usize,
        pixel_x: u32,
        pixel_y: u32,
    ) -> (u8, u8, u8, u8) {
        let xc = center - pixel_x as f64;
        let yc = center - pixel_y as f64;
        let r = {
            let dist = (xc * xc + yc * yc).sqrt();
            let rate = speed * 2.5 * nframes as f64;
            let cos = (pi2 * (dist - rate) / period).cos();
            255. * 0.5 * (1.0 + cos)
        } as u8;
        let g = {
            let dist = (xc * xc + yc * yc).sqrt();
            // let dist = xc * xc + yc * yc;
            let rate = speed * -0.5 * nframes as f64;
            let cos = (pi2 * (dist - rate) / period).sin();
            150. * 0.5 * (1.0 + cos)
        } as u8;
        let b = {
            let dist = (xc * xc + yc * yc).sqrt();
            let rate = speed * nframes as f64;
            let sin = (pi2 * (dist - rate) / period).sin();
            200. * 0.5 * (1.0 + sin)
        } as u8;

        (r, g, b, 255)
    }
}
