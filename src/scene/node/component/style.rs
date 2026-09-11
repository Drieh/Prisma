use std::time::{Duration, Instant};

use crate::util::{Color, Size};

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

#[derive(Debug)]
enum CurrentStyle {
    Normal,
    Hover,
    Active,
}

pub struct NodeStyle {
    normal: Style,
    hover: Style,
    active: Style,
    current: CurrentStyle,
}

impl NodeStyle {
    pub fn new() -> Self {
        Self {
            normal: Style::new(),
            hover: Style::new(),
            active: Style::new(),
            current: CurrentStyle::Normal,
        }
    }

    fn current_style(&self) -> &Style {
        match self.current {
            CurrentStyle::Normal => &self.normal,
            CurrentStyle::Hover => &self.hover,
            CurrentStyle::Active => &self.active,
        }
    }

    fn current_style_mut(&mut self) -> &mut Style {
        match self.current {
            CurrentStyle::Normal => &mut self.normal,
            CurrentStyle::Hover => &mut self.hover,
            CurrentStyle::Active => &mut self.active,
        }
    }

    fn modify_current_style<T>(&mut self, mut callback: T)
    where
        T: FnMut(&mut Style),
    {
        match self.current {
            CurrentStyle::Normal => {
                callback(&mut self.normal);
                callback(&mut self.hover);
                callback(&mut self.active);
            }
            CurrentStyle::Hover => {
                callback(&mut self.hover);
                callback(&mut self.active);
            }
            CurrentStyle::Active => {
                callback(&mut self.active);
            }
        }
    }

    pub fn get_color(&self) -> Color {
        self.current_style().color
    }

    pub fn get_border_radius(&self) -> u32 {
        self.current_style().border_radius
    }

    pub fn get_timer(&self) -> Option<Duration> {
        self.current_style().timer
    }

    pub fn get_timer_start(&self) -> Option<Instant> {
        self.current_style().timer_start
    }

    pub fn get_size(&self) -> Size {
        self.current_style().size
    }

    pub fn set_color(&mut self, color: Color) {
        self.modify_current_style(|style| {
            style.color = color;
        });
    }

    pub fn set_border_radius(&mut self, radius: u32) {
        self.modify_current_style(|style| {
            style.border_radius = radius;
        });
    }

    pub fn set_timer(&mut self, ms: u64) {
        self.current_style_mut().timer = Some(Duration::from_millis(ms));
        self.current_style_mut().timer_start = Some(Instant::now());
    }

    pub fn set_size(&mut self, size: Size) {
        self.modify_current_style(|style| {
            style.size = size;
        });
    }

    pub fn clear_timer(&mut self) {
        self.current_style_mut().timer = None;
        self.current_style_mut().timer_start = None;
    }

    pub fn set_normal(&mut self) {
        self.current = CurrentStyle::Normal;
    }

    pub fn set_hover(&mut self) {
        self.current = CurrentStyle::Hover;
    }

    pub fn set_active(&mut self) {
        self.current = CurrentStyle::Active;
    }
}
