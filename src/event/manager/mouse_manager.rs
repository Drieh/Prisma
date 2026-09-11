use sdl3::event::Event as SdlEvent;
use sdl3::mouse::MouseButton as SdlMouseButton;
use std::collections::{HashMap, HashSet};

use crate::{
    event::{
        Click, Drag, DragEnd, DragStart, MouseButton, MouseDown, MouseEvent, MouseMove, MouseUp,
    },
    util::Position,
};

const DRAG_TOLERANCE: i32 = 5;
pub struct MouseManager {
    position: Position,
    last_down_position: HashMap<MouseButton, Position>,
    is_dragging: HashSet<MouseButton>,
    queue: Vec<MouseEvent>,
}
impl MouseManager {
    pub fn new() -> Self {
        Self {
            is_dragging: HashSet::new(),
            position: Position { x: 0, y: 0 },
            last_down_position: HashMap::new(),
            queue: Vec::new(),
        }
    }

    fn match_sdl_mouse_button(&self, button: SdlMouseButton) -> MouseButton {
        match button {
            SdlMouseButton::Left => MouseButton::Left,
            SdlMouseButton::Middle => MouseButton::Middle,
            SdlMouseButton::Right => MouseButton::Right,
            SdlMouseButton::X1 => MouseButton::X1,
            SdlMouseButton::X2 => MouseButton::X2,
            SdlMouseButton::Unknown => MouseButton::Unknown,
        }
    }

    pub fn handle_sdl_event(&mut self, event: &SdlEvent) {
        match *event {
            SdlEvent::MouseButtonDown {
                mouse_btn: sdl_mouse_btn,
                x,
                y,
                ..
            } => {
                let mouse_btn = self.match_sdl_mouse_button(sdl_mouse_btn);
                let position = Position {
                    x: x.round() as i32,
                    y: y.round() as i32,
                };
                self.position = position;
                self.last_down_position.insert(mouse_btn, position);

                self.queue.push(MouseEvent::MouseDown(MouseDown {
                    position,
                    mouse_btn,
                }));
            }

            SdlEvent::MouseButtonUp {
                mouse_btn: sdl_mouse_btn,
                x,
                y,
                ..
            } => {
                let mouse_btn = self.match_sdl_mouse_button(sdl_mouse_btn);
                let position = Position {
                    x: x.round() as i32,
                    y: y.round() as i32,
                };
                if self.is_dragging.contains(&mouse_btn) {
                    self.queue.push(MouseEvent::MouseDragEnd(DragEnd {
                        position,
                        mouse_btn,
                    }));
                } else if self.last_down_position.contains_key(&mouse_btn) {
                    self.queue.push(MouseEvent::MouseClick(Click {
                        position,
                        mouse_btn,
                    }));
                }
                self.is_dragging.remove(&mouse_btn);
                self.last_down_position.remove(&mouse_btn);

                self.queue.push(MouseEvent::MouseUp(MouseUp {
                    position,
                    mouse_btn,
                }));
                self.position = position;
            }

            SdlEvent::MouseMotion { x, y, .. } => {
                let position = Position {
                    x: x.round() as i32,
                    y: y.round() as i32,
                };
                self.position = position;
                self.queue
                    .push(MouseEvent::MouseMove(MouseMove { position }));
                for (button, down_position) in &self.last_down_position {
                    let Position {
                        x: down_position_x,
                        y: down_position_y,
                    } = *down_position;

                    let dx = position.x - down_position_x;
                    let dy = position.y - down_position_y;

                    if dx * dx + dy * dy > DRAG_TOLERANCE * DRAG_TOLERANCE {
                        if self.is_dragging.contains(button) {
                            self.queue.push(MouseEvent::MouseDrag(Drag {
                                position,
                                mouse_btn: *button,
                            }));
                        } else {
                            self.is_dragging.insert(*button);
                            self.queue.push(MouseEvent::MouseDragStart(DragStart {
                                position: Position {
                                    x: down_position_x,
                                    y: down_position_y,
                                },
                                mouse_btn: *button,
                            }));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn take_events(&mut self) -> Vec<MouseEvent> {
        std::mem::take(&mut self.queue)
    }
}
