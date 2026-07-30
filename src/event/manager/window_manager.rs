use sdl3::event::Event as SdlEvent;
use sdl3::event::WindowEvent as SdlWindowEvent;

use crate::event::WindowCloseRequest;
use crate::event::WindowEvent;
use crate::event::WindowMove;
use crate::event::WindowResized;
use crate::util::Position;

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
                    self.queue
                        .push(WindowEvent::WindowCloseRequest(WindowCloseRequest {
                            window_id: *window_id,
                        }));
                }
                SdlWindowEvent::Moved(x, y) => {
                    self.queue.push(WindowEvent::WindowMove(WindowMove {
                        window_id: *window_id,
                        position: Position {
                            x: *x as f32,
                            y: *y as f32,
                        },
                    }));
                }
                SdlWindowEvent::Resized(w, h) => {
                    self.queue.push(WindowEvent::WindowResized(WindowResized {
                        window_id: *window_id,
                        width: *w,
                        height: *h,
                    }));
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
