use sdl2::render::Texture;
use std::sync::{mpsc::Sender, Arc, Mutex};
// Holds all the drawing logic, like the graph rendering and the settings display
// Also parses text and makes equation logic

pub fn main_loop(
    (main_texture, graphing_texture, sender): (
        Arc<Mutex<Texture>>,
        Arc<Mutex<Texture>>,
        Sender<bool>,
    ),
) -> ! {
    '_main: loop {
        
    }
}
