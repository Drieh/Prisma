use std::{
    ops::Add,
    time::{Duration, Instant},
};

use crate::util::{Color, Position, Scale, Size};

#[derive(Clone, Copy)]
struct Transform {
    pub position: Position,
    pub position_absolute: bool,
    pub rotation: f32,
    pub scale: Scale,
    pub layer: Option<usize>,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Position::new(),
            position_absolute: false,
            rotation: 0.0,
            scale: Scale::new(),
            layer: None,
        }
    }
}

impl Add for Transform {
    type Output = Transform;
    fn add(self, rhs: Self) -> Self::Output {
        Transform {
            position: self.position + rhs.position,
            position_absolute: self.position_absolute,
            rotation: self.rotation + rhs.rotation,
            scale: Scale {
                x: self.scale.x * rhs.scale.x,
                y: self.scale.y * rhs.scale.y,
            },
            layer: rhs.layer.or(self.layer),
        }
    }
}

impl Add for &Transform {
    type Output = Transform;
    fn add(self, rhs: &Transform) -> Transform {
        *self + *rhs
    }
}

#[derive(Clone, Copy)]
struct Style {
    color: Color,
    border_radius: u32,
    size: Size,
    timer: Option<Duration>,
    timer_start: Option<Instant>,
}

impl Style {
    pub fn new() -> Self {
        Self {
            color: Color::TRANSPARENT,
            border_radius: 0,
            size: Size::new(),
            timer: None,
            timer_start: None,
        }
    }
}

#[derive(Clone, Copy)]
struct VisualState {
    transform: Transform,
    style: Style,
}

impl VisualState {
    fn new() -> Self {
        Self {
            transform: Transform::new(),
            style: Style::new(),
        }
    }
}

#[derive(Clone, Copy)]
enum CurrentVisualState {
    Normal,
    Hover,
    Active,
}

#[derive(Clone, Copy)]
pub struct NodeVisual {
    normal: VisualState,
    hover: VisualState,
    active: VisualState,
    current: CurrentVisualState,
}

impl NodeVisual {
    pub fn new() -> Self {
        Self {
            normal: VisualState::new(),
            hover: VisualState::new(),
            active: VisualState::new(),
            current: CurrentVisualState::Normal,
        }
    }

    fn get_current_state(&self) -> &VisualState {
        match self.current {
            CurrentVisualState::Normal => &self.normal,
            CurrentVisualState::Hover => &self.hover,
            CurrentVisualState::Active => &self.active,
        }
    }

    fn current_style_mut(&mut self) -> &mut VisualState {
        match self.current {
            CurrentVisualState::Normal => &mut self.normal,
            CurrentVisualState::Hover => &mut self.hover,
            CurrentVisualState::Active => &mut self.active,
        }
    }

    fn fusion_state<T>(&mut self, mut callback: T)
    where
        T: FnMut(&mut VisualState),
    {
        match self.current {
            CurrentVisualState::Normal => {
                callback(&mut self.normal);
                callback(&mut self.hover);
                callback(&mut self.active);
            }
            CurrentVisualState::Hover => {
                callback(&mut self.hover);
                callback(&mut self.active);
            }
            CurrentVisualState::Active => {
                callback(&mut self.active);
            }
        }
    }

    pub fn get_color(&self) -> Color {
        self.get_current_state().style.color
    }

    pub fn get_border_radius(&self) -> u32 {
        self.get_current_state().style.border_radius
    }

    pub fn get_timer(&self) -> Option<Duration> {
        self.get_current_state().style.timer
    }

    pub fn get_timer_start(&self) -> Option<Instant> {
        self.get_current_state().style.timer_start
    }

    pub fn get_size(&self) -> Size {
        self.get_current_state().style.size
    }

    pub fn get_relative_position(&self) -> Position {
        self.get_current_state().transform.position
    }

    pub fn is_position_absolute(&self) -> bool {
        self.get_current_state().transform.position_absolute
    }

    pub fn get_rotation(&self) -> f32 {
        self.get_current_state().transform.rotation
    }

    pub fn get_scale(&self) -> Scale {
        self.get_current_state().transform.scale
    }

    pub fn get_layer(&self) -> Option<usize> {
        self.get_current_state().transform.layer
    }

    pub fn get_bounding_box(&self) -> Size {
        Size {
            width: (self.get_size().width as f32 * self.get_scale().x)
                .abs()
                .round() as u32,
            height: (self.get_size().height as f32 * self.get_scale().y)
                .abs()
                .round() as u32,
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.fusion_state(|state| {
            state.style.color = color;
        });
    }

    pub fn set_border_radius(&mut self, radius: u32) {
        self.fusion_state(|state| {
            state.style.border_radius = radius;
        });
    }

    pub fn set_timer(&mut self, ms: u64) {
        self.current_style_mut().style.timer = Some(Duration::from_millis(ms));
        self.current_style_mut().style.timer_start = Some(Instant::now());
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.fusion_state(|state| {
            state.style.size = Size { width, height };
        });
    }

    pub fn set_position(&mut self, position: Position) {
        self.fusion_state(|state| {
            state.transform.position = position;
        });
    }

    pub fn set_position_absolute(&mut self) {
        self.fusion_state(|state| {
            state.transform.position_absolute = true;
        });
    }

    pub fn set_position_relative(&mut self) {
        self.fusion_state(|state| {
            state.transform.position_absolute = false;
        });
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.fusion_state(|state| {
            state.transform.rotation = rotation;
        });
    }

    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.fusion_state(|state| {
            state.transform.scale = Scale { x, y };
        });
    }

    pub fn set_layer(&mut self, layer: usize) {
        self.fusion_state(|state| {
            state.transform.layer = Some(layer);
        });
    }

    pub fn clear_timer(&mut self) {
        self.current_style_mut().style.timer = None;
        self.current_style_mut().style.timer_start = None;
    }

    pub fn set_normal(&mut self) {
        self.current = CurrentVisualState::Normal;
    }

    pub fn set_hover(&mut self) {
        self.current = CurrentVisualState::Hover;
    }

    pub fn set_active(&mut self) {
        self.current = CurrentVisualState::Active;
    }
}
