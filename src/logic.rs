use super::windows::SafeTexture;
use sdl2::render::TextureQuery;
use std::alloc::{alloc, dealloc, Layout, LayoutError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc::Receiver, Arc};
use std::thread::{spawn, JoinHandle};
// Holds all the drawing logic, like the graph rendering and the settings display
// Also parses text and makes equation logic

pub fn main_loop(
    (main_lock, graphing_lock, sender): (Arc<SafeTexture>, Arc<SafeTexture>, Receiver<bool>),
    monitor: Arc<AtomicBool>,
    width: u32,
    height: u32,
) -> Result<(), String> {
    // Initialize all variables
    let cores = num_cpus::get();
    // let cores = 3usize;
    let mut nframes = 0;
    let mut pixels = Pixels::new((width * height * 4) as usize).map_err(|e| e.to_string())?;
    let splits = {
        let dist = ((width * height) as usize / cores) * 4;
        let mut vec = Vec::with_capacity(cores as usize);
        for i in 0..cores {
            vec.push(i * dist);
        }
        vec
    };
    let mut slices = match unsafe { pixels.split_at_mut_unsafe(&splits[1..]) } {
        Some(s) => s,
        None => return Err("Failed to make pixel slices".to_string()),
    };
    let data = get_draw_data(width, height);
    let mut handles = Vec::with_capacity(cores);
    // Start main loop
    while monitor.load(Ordering::Relaxed) {
        // Wait for the window to give up control on the textures
        match sender.recv() {
            Ok(true) => (),
            Ok(false) => return Ok(()),
            Err(_) => {
                return Err("Something happened with the transmitter in main window!".to_string())
            }
        };
        // Grab the texture's lock again, and use them
        {
            let main_texture = match main_lock.lock() {
                Ok(t) => t,
                Err(_) => return Err("The main mutex was poisoned!".to_string()),
            };
            let mut graphing_texture = match graphing_lock.lock() {
                Ok(t) => t,
                Err(_) => return Err("The main mutex was poisoned!".to_string()),
            };
            // Do drawing on the pixels buffer
            for i in 0..cores {
                let slice = slices[i].take().ok_or("Unreachable".to_string())?;
                let ind = splits[i];
                let pitch = width as usize * 4;
                let handle = spawn(move || {
                    draw_func(slice, &data, nframes, ind, pitch);
                });
                handles.push(handle);
            }
            while !handles.is_empty() {
                handles
                    .pop()
                    .expect("Unreachable")
                    .join()
                    .expect("Child Thread Panicked");
            }
            // Copy buffer to the graphing texture
            let slice = pixels.as_slice();
            graphing_texture
                .update(None, slice, (width * 4) as usize)
                .map_err(|e| e.to_string())?;
            // Remake slices
            slices = match unsafe { pixels.split_at_mut_unsafe(&splits[1..]) } {
                Some(s) => s,
                None => return Err("Failed to make pixel slices".to_string()),
            };
        }
        // Once we finished using the textures, presumably
        // the main thread will update the screen and send us a message
        nframes += 1;
    }
    drop(slices);
    Ok(())
}

fn draw_func(
    slice: &mut [u8],
    data: &(f64, f64, f64, f64),
    nframes: usize,
    ind: usize,
    pitch: usize,
) {
    for i in 0..slice.len() / 4 {
        let j = i * 4 + ind;
        let (x, y) = {
            let x = (j % pitch) / 4;
            let y = j / pitch;
            (x as u32, y as u32)
        };
        let (r, g, b) = draw_pixel(data, nframes, x, y);
        slice[i * 4 + 0] = r;
        slice[i * 4 + 1] = g;
        slice[i * 4 + 2] = b;
        slice[i * 4 + 3] = 255;
    }
}

fn get_draw_data(width: u32, height: u32) -> (f64, f64, f64, f64) {
    let center = width as f64 / 2.;
    let speed = width as f64 / 500.;
    let period = width as f64 / 10.;
    let pi2 = std::f64::consts::PI * 2.;
    (center, speed, period, pi2)
}

fn draw_pixel(
    (center, speed, period, pi2): &(f64, f64, f64, f64),
    nframes: usize,
    x: u32,
    y: u32,
) -> (u8, u8, u8) {
    let xc = center - x as f64;
    let yc = center - y as f64;
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

    (r, g, b)
}

pub struct Pixels {
    ptr: *mut u8,
    len: usize,
}

impl Pixels {
    pub fn new(len: usize) -> Result<Self, LayoutError> {
        let ptr = unsafe {
            let layout = Layout::array::<u8>(len)?;
            alloc(layout) as *mut u8
        };
        Ok(Self { ptr, len })
    }
    pub fn get(&self, idx: usize) -> Option<&u8> {
        if idx < self.len {
            unsafe { Some(&*(self.ptr.add(idx))) }
        } else {
            None
        }
    }
    pub fn get_mut(&self, idx: usize) -> Option<&mut u8> {
        if idx < self.len {
            unsafe { Some(&mut *(self.ptr.add(idx))) }
        } else {
            None
        }
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

impl std::ops::Index<usize> for Pixels {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}
impl std::ops::IndexMut<usize> for Pixels {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
