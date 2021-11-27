use crate::rendering::RenderOpReference;
use crate::windows::Window;
use sdl2::event::{Event, EventWatch, EventWatchCallback, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::EventSubsystem;
use std::sync::{mpsc::Sender, Arc, Mutex};

pub enum MainEvent {
    Quit(Result<(), String>),
    RenderOpStart(RenderOpReference),
    RenderOpFinish(RenderOpReference),
}

pub enum SDL_Event {
    Event(Event),
    User(MainEvent),
}

pub struct EventWatcher {
    render_ops: Vec<RenderOpReference>,
    windows: Vec<Arc<Mutex<Window>>>,
    transmitter: Sender<MainEvent>,
}

impl EventWatcher {
    pub fn init(
        ev: &EventSubsystem,
        transmitter: Sender<MainEvent>,
        windows: Vec<Arc<Mutex<Window>>>,
        render_ops: Vec<RenderOpReference>,
    ) -> EventWatch<'_, Self> {
        let mut ws = vec![];
        for window in windows {
            ws.push(window);
        }
        let windows = ws;
        let event_watcher = Self {
            render_ops,
            windows,
            transmitter,
        };

        ev.add_event_watch(event_watcher)
    }
}

impl EventWatchCallback for EventWatcher {
    fn callback(&mut self, event: Event) {
        let this_event;
        if let Event::User { .. } = event {
            let user = event.as_user_event_type::<MainEvent>().unwrap();
            this_event = SDL_Event::User(user);
        } else {
            this_event = SDL_Event::Event(event);
        }
        let event = this_event;
        // Handle events
        match &event {
            SDL_Event::Event(event) => match event {
                // If sdl2 wants to quite or escape is pressed,
                // Then quit
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    self.transmitter.send(MainEvent::Quit(Ok(()))).unwrap();
                }
                // If a window resizes, then we need to tell it
                Event::Window {
                    win_event: WindowEvent::Resized(wid, hei),
                    window_id: id,
                    ..
                } => {
                    for window in &self.windows {
                        let mut window = window.lock().unwrap();
                        if *id == window.canvas().window().id() {
                            window.resized(*wid as usize, *hei as usize).unwrap();
                            return;
                        }
                    }
                    // Failed to find window, so quit
                    self.transmitter
                        .send(MainEvent::Quit(Err("Window Resize Event Fail".to_string())))
                        .unwrap();
                }
                // Default, do nothing
                _ => (),
            },
            SDL_Event::User(event) => match event {
                MainEvent::RenderOpFinish(op) => {
                    self.transmitter
                        .send(MainEvent::RenderOpFinish(op.clone()))
                        .unwrap();
                }
                _ => (),
            },
        }
        // Send events to ops and tell them to handle
        // if applicable
        for op_ref in &self.render_ops {
            let op = op_ref.read().unwrap();
            op.push_event(event);
            if op.get_open() {
                let should_start = op.handle_events();
                if should_start {
                    self.transmitter
                        .send(MainEvent::RenderOpStart(op_ref.clone()))
                        .unwrap();
                }
            }
            return;
        }
    }
}
