use super::windows::SafeTexture;
use std::sync::{mpsc::Receiver, Arc};
// Holds all the drawing logic, like the graph rendering and the settings display
// Also parses text and makes equation logic

pub fn main_loop(
    (main_texture, graphing_texture, sender): (Arc<SafeTexture>, Arc<SafeTexture>, Receiver<bool>),
) -> ! {
    '_main: loop {}
}
