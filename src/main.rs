pub fn main() -> Result<(), String> {
    // Call setup functions for sdl2

    // Call Main Window Init and Open Function from windows.rs

    // Call Settings Window Init from windows.rs

    // Call All Other Window Init Functions from windows.rs

    // Make a new thread to handle the drawing logic in logic.rs

    // Make a new thread to handle and process all events,
    // Sending the data off to the required threads, covered in events.rs

    // Start the event loop and pass the events to the proper threads,
    // As well as wait for any draw requests from the drawing thread

    // Triangle of messaging
    /*
                   Events with
                   Drawing Impacts
         logic.rs <------- events.rs
            \                  >
             \               /
    Updated   \            /   All Events I want to handle
    Frames     \         /
                >      /
                 main.rs

     */

    Ok(())
}

fn test(i: usize) {
    let arr = vec![[0; 3]; i];
}
