use sdl3::event::Event as SdlEvent;
use sdl3::event::WindowEvent as SdlWindowEvent;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum WindowEventType {
    Move,
    Resized,
    Minimized,
    Maximized,
    CloseRequest,
}
#[derive(Clone, Copy, Debug)]
pub enum WindowEvent {
    Move { window_id: u32, x: i32, y: i32 },
    Resized { width: i32, height: i32 },
    Minimized { x: f32, y: f32 },
    Maximized { x: f32, y: f32 },
    CloseRequest,
}
impl WindowEvent {
    pub fn event_type(&self) -> WindowEventType {
        match self {
            WindowEvent::Move { .. } => WindowEventType::Move,
            WindowEvent::Resized { .. } => WindowEventType::Resized,
            WindowEvent::Minimized { .. } => WindowEventType::Minimized,
            WindowEvent::Maximized { .. } => WindowEventType::Maximized,
            WindowEvent::CloseRequest { .. } => WindowEventType::CloseRequest,
        }
    }
}

pub struct WindowManager {
    close_requested: bool,
    queue: Vec<WindowEvent>,
}
impl WindowManager {
    pub fn new() -> Self {
        Self {
            close_requested: false,
            queue: Vec::new(),
        }
    }

    pub fn handle_sdl_event(&mut self, event: &SdlEvent) {
        if let SdlEvent::Window {
            window_id,
            win_event,
            ..
        } = event
        {
            match win_event {
                SdlWindowEvent::CloseRequested => {
                    self.close_requested = true;
                    self.queue.push(WindowEvent::CloseRequest);
                }
                SdlWindowEvent::Moved(x, y) => {
                    self.queue.push(WindowEvent::Move {
                        window_id: *window_id,
                        x: *x,
                        y: *y,
                    });
                }
                SdlWindowEvent::Resized(w, h) => {
                    self.queue.push(WindowEvent::Resized {
                        width: *w,
                        height: *h,
                    });
                }

                _ => {}
            }
        }
    }

    pub fn take_events(&mut self) -> Vec<WindowEvent> {
        //self.create_derived_events();
        std::mem::take(&mut self.queue)
    }
}
