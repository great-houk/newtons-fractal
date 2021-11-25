// Holds all the drawing logic, like the graph rendering and the settings display
pub use render_backend::{main::main_loop, RenderOp};

mod render_backend {
    type Pixel = (u8, u8, u8, u8);
    type PixelSlice<'a> = (&'a mut [Pixel], usize, usize);
    type RenderOpReference = &'static Box<dyn RenderOp<Data = Box<dyn Sync>>>;

    pub trait RenderOp: Sync {
        type Data: Sync;
        fn get_pixels(&mut self) -> &mut Pixels;
        fn get_slice<'a>(&mut self, ind: usize, max: usize) -> PixelSlice<'a>;
        fn draw_pixel(&self, x: usize, y: usize) -> Pixel;
        fn init_data(&mut self);
        fn get_data(&self) -> &Self::Data;
        fn modify_data(&self);
    }

    pub enum ThreadMessage {
        Op(RenderOpReference),
        Resize { width: usize, height: usize },
        Quit,
    }

    pub mod main {
        use super::threading::{end_threads, start_threads};
        use super::{RenderOpReference, ThreadMessage};
        use std::sync::mpsc::{Receiver, Sender};

        pub fn main_loop(
            (sender, receiver): (Sender<RenderOpReference>, Receiver<ThreadMessage>),
        ) -> Result<(), String> {
            // Initialize all variables
            let cores = num_cpus::get() * 2;
            let (mut handles, mut senders, mut receivers) = start_threads(cores)?;
            // Start main loop
            'main: loop {
                // Wait for the window to give up control on the textures
                match receiver.recv() {
                    Ok(ThreadMessage::Op(op)) => {
                        // Start Render
                        for sender in &senders {
                            sender.send(Some(op)).unwrap();
                        }
                        // Wait for threads to finish
                        for receiver in &receivers {
                            receiver.recv().unwrap();
                        }
                        // Allow drawing logic to modify data
                        let x = op.as_ref();
                        x.modify_data();
                        // Let main thread know we're done
                        sender
                            .send(op)
                            .expect("The Main Thread's Receiver is Dead!");
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
                receivers.push(rx);
                senders.push(tx);
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
        use super::{Pixels, RenderOpReference};
        use std::sync::mpsc::{Receiver, Sender};

        pub fn draw_loop(
            sender: Sender<()>,
            receiver: Receiver<Option<RenderOpReference>>,
            id: usize,
            thread_count: usize,
        ) {
            while let Ok(Some(op)) = receiver.recv() {
                // Get slice
                let (slice, ind, pitch) = op.get_slice(id, thread_count);
                // Modify pixels
                draw_func(op, slice, ind, pitch);
                // Tell logic thread we're done
                sender.send(()).unwrap();
            }
        }

        fn draw_func(
            op: RenderOpReference,
            slice: &mut [(u8, u8, u8, u8)],
            ind: usize,
            pitch: usize,
        ) {
            for i in 0..slice.len() {
                let total_ind = i + ind;
                let (pixel_x, pixel_y) = {
                    let x = total_ind % pitch;
                    let y = total_ind / pitch;
                    (x, y)
                };
                let color = op.draw_pixel(pixel_x, pixel_y);
                slice[i] = color;
            }
        }
    }
}
