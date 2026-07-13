use sdl3::event::Event as SdlEvent;
use sdl3::mouse::MouseButton as SdlMouseButton;
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use crate::common::Position;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
    Unknown,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum MouseEventType {
    Move,
    Down,
    Up,
    Click,
    Hover,
    Active,
    Enter,
    Leave,
}
#[derive(Clone, Copy, Debug)]
pub enum MouseEvent {
    MouseMove {
        x: f32,
        y: f32,
    },
    MouseDown {
        x: f32,
        y: f32,
        mouse_btn: MouseButton,
    },
    MouseUp {
        x: f32,
        y: f32,
        mouse_btn: MouseButton,
    },
    Click {
        x: f32,
        y: f32,
        mouse_btn: MouseButton,
    },
}
impl MouseEvent {
    pub fn event_type(&self) -> MouseEventType {
        match self {
            MouseEvent::MouseMove { .. } => MouseEventType::Move,
            MouseEvent::MouseDown { .. } => MouseEventType::Down,
            MouseEvent::MouseUp { .. } => MouseEventType::Up,
            MouseEvent::Click { .. } => MouseEventType::Click,
        }
    }
}

pub struct MouseManager {
    position: Position,
    last_down_position: HashMap<MouseButton, (f32, f32, Instant)>,
    pressed_buttons: HashSet<MouseButton>,
    queue: Vec<MouseEvent>,
}

impl MouseManager {
    pub fn new() -> Self {
        Self {
            position: Position { x: 0.0, y: 0.0 },
            last_down_position: HashMap::new(),
            pressed_buttons: HashSet::new(),
            queue: Vec::new(),
        }
    }

    fn match_sdl_mouse_button(&self, button: SdlMouseButton) -> MouseButton {
        match button {
            SdlMouseButton::Left => {
                return MouseButton::Left;
            }
            SdlMouseButton::Middle => {
                return MouseButton::Middle;
            }
            SdlMouseButton::Right => {
                return MouseButton::Right;
            }
            SdlMouseButton::X1 => {
                return MouseButton::X1;
            }
            SdlMouseButton::X2 => {
                return MouseButton::X2;
            }
            SdlMouseButton::Unknown => {
                return MouseButton::Unknown;
            }
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
                self.position = Position { x, y };
                self.pressed_buttons.insert(mouse_btn);
                self.last_down_position
                    .insert(mouse_btn, (x, y, Instant::now()));

                self.queue.push(MouseEvent::MouseDown { x, y, mouse_btn });
            }

            SdlEvent::MouseButtonUp {
                mouse_btn: sdl_mouse_btn,
                x,
                y,
                ..
            } => {
                let mouse_btn = self.match_sdl_mouse_button(sdl_mouse_btn);
                self.position = Position { x, y };
                self.pressed_buttons.remove(&mouse_btn);
                self.queue.push(MouseEvent::MouseUp { x, y, mouse_btn });
            }

            SdlEvent::MouseMotion { x, y, .. } => {
                self.position = Position { x, y };
                self.queue.push(MouseEvent::MouseMove { x, y });
            }

            _ => {}
        }
    }

    // Genera Click, Drag, etc.
    fn create_derived_events(&mut self) {
        const CLICK_TOLERANCE: f32 = 5.0;

        let events = std::mem::take(&mut self.queue);

        for event in events {
            match event {
                MouseEvent::MouseUp { x, y, mouse_btn } => {
                    self.queue.push(MouseEvent::MouseUp { x, y, mouse_btn });

                    if let Some((down_x, down_y, instant)) =
                        self.last_down_position.remove(&mouse_btn)
                    {
                        let dx = x - down_x;
                        let dy = y - down_y;

                        if dx * dx + dy * dy <= CLICK_TOLERANCE * CLICK_TOLERANCE
                            && instant.elapsed() < Duration::from_millis(600)
                        {
                            self.queue.push(MouseEvent::Click { x, y, mouse_btn });
                        }
                    }
                }

                other => {
                    self.queue.push(other);
                }
            }
        }
    }
    // Procesa MouseDown
    fn handle_mouse_down(&mut self) {}

    // Procesa MouseUp
    fn handle_mouse_up(&mut self) {}

    // Procesa MouseMove
    fn handle_mouse_move(&mut self) {}

    // Devuelve la cola para que EventManager la despache
    pub fn take_events(&mut self) -> Vec<MouseEvent> {
        self.create_derived_events();
        std::mem::take(&mut self.queue)
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }
}
