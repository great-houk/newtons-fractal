use crate::rendering::RenderOpReference;
use crate::windows::Window;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use sdl2::Sdl;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub enum MainEvent {
    Quit(Result<(), String>),
    RenderOpStart(RenderOpReference),
    RenderOpFinish(RenderOpReference),
}

#[derive(Clone)]
pub enum SDL_Event {
    Event(Event),
    User(MainEvent),
}

pub struct EventHandler {
    event_pump: EventPump,
    windows: Vec<Arc<Mutex<Window>>>,
    render_ops: Vec<RenderOpReference>,
}

impl EventHandler {
    pub fn init(
        ev: &Sdl,
        windows: Vec<Arc<Mutex<Window>>>,
        render_ops: Vec<RenderOpReference>,
    ) -> Result<Self, String> {
        let mut ws = vec![];
        for window in windows {
            ws.push(window);
        }
        let windows = ws;
        let event_pump = ev.event_pump()?;
        Ok(EventHandler {
            event_pump,
            windows,
            render_ops,
        })
    }

    pub fn handle_events(&mut self) -> Vec<MainEvent> {
        let mut ret = vec![];
        let event = self.event_pump.wait_event();
        // Change event into sdl_event b/c as_user_event_type
        // can only be done once.
        let event = Self::transform_event(event);
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
                    ret.push(MainEvent::Quit(Ok(())));
                }
                // If a window resizes, then we need to tell it
                Event::Window {
                    win_event: WindowEvent::Resized(wid, hei),
                    window_id: id,
                    ..
                } => {
                    let mut found = false;
                    for window in &self.windows {
                        let mut window = window.lock().unwrap();
                        if *id == window.canvas().window().id() {
                            window.resized(*wid as usize, *hei as usize).unwrap();
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        // Failed to find window, so quit
                        ret.push(MainEvent::Quit(Err("Window Resize Event Fail".to_string())));
                    }
                }
                // Default, do nothing
                _ => (),
            },
            SDL_Event::User(event) => match event {
                MainEvent::RenderOpFinish(op) => {
                    ret.push(MainEvent::RenderOpFinish(op.clone()));
                }
                _ => (),
            },
        }
        // Send events to ops and tell them to handle
        // if applicable
        for op_ref in &self.render_ops {
            let op = op_ref.read().unwrap();
            op.push_event(event.clone());
            if op.get_open() {
                let should_start = op.handle_events();
                if should_start {
                    ret.push(MainEvent::RenderOpStart(op_ref.clone()));
                }
            }
        }
        ret
    }

    fn transform_event(event: Event) -> SDL_Event {
        match event {
            Event::User { .. } => SDL_Event::User(event.as_user_event_type::<MainEvent>().unwrap()),
            _ => SDL_Event::Event(event),
        }
    }
}
