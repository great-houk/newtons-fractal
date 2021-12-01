// Holds all the drawing logic, like the graph rendering and the settings display
pub use render_backend::{
    main::main_loop, pixels::Pixels, Pixel, PixelSlice, RenderOp, RenderOpReference, ThreadMessage,
};

mod render_backend {
    use crate::events::SdlEvent;
    use crate::windows::Window;
    use pixels::Pixels;
    use sdl2::rect::Rect;
    use std::sync::{Arc, Mutex, RwLock};
    pub type Pixel = (u8, u8, u8, u8);
    pub type PixelSlice<'a> = (&'a mut [Pixel], usize, usize);
    pub type RenderOpReference = Arc<RwLock<Box<dyn RenderOp + Send>>>;

    pub trait RenderOp: Sync {
        fn get_window(&self) -> Arc<Mutex<Window>>;
        fn get_rect(&self) -> &Rect;
        fn get_present_buffer(&self) -> &Pixels;
        fn get_draw_buffer(&self) -> &Pixels;
        fn swap_buffers(&mut self);
        fn get_slice<'a>(&self, ind: usize, max: usize) -> PixelSlice<'a> {
            let pixels = self.get_draw_buffer();
            let (slice, ind) = unsafe { pixels.get_slice(ind, max) };
            let pitch = pixels.dimensions().0;
            (slice, ind, pitch)
        }
        fn draw_pixel(&self, x: usize, y: usize) -> Pixel;
        fn modify_data(&mut self);
        fn handle_events(&mut self) -> bool;
        fn push_event(&self, event: SdlEvent);
        fn set_open(&self, state: bool);
        fn get_open(&self) -> bool;
    }

    pub enum ThreadMessage {
        Op(RenderOpReference),
        Quit,
    }

    pub mod main {
        use super::threading::{end_threads, start_threads};
        use super::ThreadMessage;
        use crate::events::MainEvent;
        use sdl2::event::EventSender;
        use std::sync::mpsc::Receiver;

        pub fn main_loop(
            sender: EventSender,
            receiver: Receiver<ThreadMessage>,
        ) -> Result<(), String> {
            // Initialize all variables
            let cores = num_cpus::get() * 2;
            let (handles, senders, receivers) = start_threads(cores)?;
            // Start main loop
            'main: loop {
                // Wait for the window to give up control on the textures
                match receiver.recv() {
                    Ok(ThreadMessage::Op(op)) => {
                        // Set op as closed
                        {
                            let op_read = op.read().unwrap();
                            op_read.set_open(false);
                        }
                        // Start Render
                        for sender in &senders {
                            sender.send(Some(op.clone())).unwrap();
                        }
                        // Wait for threads to finish
                        for receiver in &receivers {
                            receiver.recv().unwrap();
                        }
                        // Modify data
                        {
                            let mut op_mut = op.write().unwrap();
                            // Allow drawing logic to modify data
                            op_mut.modify_data();
                            // Swap the buffers
                            op_mut.swap_buffers();
                            // Set op to open
                            op_mut.set_open(true);
                        }
                        // Let main thread know we're done
                        sender
                            .push_custom_event(MainEvent::RenderOpFinish(op.clone()))
                            .unwrap();
                    }
                    Ok(ThreadMessage::Quit) => {
                        break 'main;
                    }
                    Err(_) => {
                        return Err(
                            "Something happened with the transmitter in main window!".to_string()
                        )
                    }
                };
            }
            // We are stopping, so all sub threads need to stop too
            end_threads(senders, handles)
        }
    }

    pub mod threading {
        use super::drawing::draw_loop;
        use super::RenderOpReference;
        use std::sync::mpsc::{channel, Receiver, Sender};
        use std::thread::{spawn, JoinHandle};

        type Thread = (
            Vec<JoinHandle<()>>,
            Vec<Sender<Option<RenderOpReference>>>,
            Vec<Receiver<()>>,
        );
        pub fn start_threads(thread_count: usize) -> Result<Thread, String> {
            let mut handles = Vec::with_capacity(thread_count);
            let mut receivers = Vec::with_capacity(thread_count);
            let mut senders = Vec::with_capacity(thread_count);
            for id in 0..thread_count {
                let (ttx, rx) = channel();
                let (tx, trx) = channel();
                let handle = spawn(move || {
                    draw_loop(ttx, trx, id, thread_count);
                });
                senders.push(tx);
                receivers.push(rx);
                handles.push(handle);
            }
            Ok((handles, senders, receivers))
        }

        pub fn end_threads(
            senders: Vec<Sender<Option<RenderOpReference>>>,
            handles: Vec<JoinHandle<()>>,
        ) -> Result<(), String> {
            for sender in senders {
                sender.send(None).map_err(|e| e.to_string())?;
            }
            for handle in handles {
                handle.join().expect("Child Thread Panicked");
            }
            Ok(())
        }
    }

    pub mod drawing {
        use super::{RenderOp, RenderOpReference};
        use std::sync::mpsc::{Receiver, Sender};

        pub fn draw_loop(
            sender: Sender<()>,
            receiver: Receiver<Option<RenderOpReference>>,
            id: usize,
            thread_count: usize,
        ) {
            while let Ok(Some(op)) = receiver.recv() {
                let op = op.read().unwrap();
                // Get slice
                let (slice, ind, pitch) = op.get_slice(id, thread_count);
                // Modify pixels
                draw_func(op.as_ref(), slice, ind, pitch);
                // Release read and let main thread know
                drop(op);
                sender.send(()).unwrap();
            }
        }

        fn draw_func(
            op: &(dyn RenderOp + Send),
            slice: &mut [(u8, u8, u8, u8)],
            ind: usize,
            pitch: usize,
        ) {
            for (i, pixel) in slice.iter_mut().enumerate() {
                let total_ind = i + ind;
                let (pixel_x, pixel_y) = {
                    let x = total_ind % pitch;
                    let y = total_ind / pitch;
                    (x, y)
                };
                let color = op.draw_pixel(pixel_x, pixel_y);
                *pixel = color;
            }
        }
    }

    pub mod pixels {
        use super::Pixel;
        use std::alloc::{alloc, dealloc, Layout, LayoutError};
        use std::slice::from_raw_parts_mut;

        pub struct Pixels {
            width: usize,
            height: usize,
            ptr: *mut u8,
            len: usize,
        }

        impl Pixels {
            pub fn new(width: usize, height: usize) -> Result<Self, LayoutError> {
                let len = 4 * width * height;
                if len == 0 {
                    return Ok(Self {
                        width,
                        height,
                        ptr: std::ptr::null_mut::<u8>(),
                        len: 0,
                    });
                }
                let ptr = unsafe {
                    let layout = Layout::array::<u8>(len)?;
                    alloc(layout) as *mut u8
                };
                Ok(Self {
                    width,
                    height,
                    ptr,
                    len,
                })
            }

            pub fn dimensions(&self) -> (usize, usize) {
                (self.width, self.height)
            }

            pub unsafe fn get_slice<'a>(&self, ind: usize, max: usize) -> (&'a mut [Pixel], usize) {
                if self.len == 0 {
                    return (&mut [], 0);
                }
                assert!(ind < max);
                let offset = 4 * ((self.len / 4) / max);
                let ptr = self.ptr.add(offset * ind);
                let len;
                if ind == max - 1 {
                    len = self.len - (offset * ind);
                } else {
                    len = offset;
                }
                (
                    from_raw_parts_mut(ptr as *mut (u8, u8, u8, u8), len / 4),
                    (offset * ind / 4),
                )
            }
        }

        impl<'a> From<&'a Pixels> for &'a [u8] {
            fn from(pixels: &'a Pixels) -> &'a [u8] {
                if pixels.len == 0 {
                    return &[];
                }
                unsafe { from_raw_parts_mut(pixels.ptr, pixels.len) }
            }
        }

        impl<'a> From<&'a Pixels> for &'a [Pixel] {
            fn from(pixels: &'a Pixels) -> &'a [Pixel] {
                if pixels.len == 0 {
                    return &[];
                }
                unsafe { from_raw_parts_mut(pixels.ptr as *mut Pixel, pixels.len / 4) }
            }
        }

        unsafe impl Send for Pixels {}
        unsafe impl Sync for Pixels {}

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
}
