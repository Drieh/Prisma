use std::{
    collections::{HashMap, HashSet},
    ops::Add,
    time::{Duration, Instant},
};

use crate::{
    node::StyleActionOrigin,
    resources::{
        FontID, TextureID,
        font::{FontStyle, TextAlign},
    },
    util::{Color, Position, Scale, Size},
};

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
    bg_color: Color,
    bg_image: Option<TextureID>,
    border_radius: u32,
    size: Size,
    timer: Option<Duration>,
    timer_start: Option<Instant>,
}

impl Style {
    pub fn new() -> Self {
        Self {
            bg_color: Color::TRANSPARENT,
            bg_image: None,
            border_radius: 0,
            size: Size::new(),
            timer: None,
            timer_start: None,
        }
    }
}

#[derive(Clone)]
pub struct Text {
    content: Option<String>,
    align: TextAlign,
    color: Color,
    font: Option<FontID>,
    font_style: FontStyle,
    font_size: u16,
}

impl Text {
    pub fn new() -> Self {
        Self {
            content: None,
            font: None,
            font_size: 16,
            font_style: FontStyle {},
            color: Color::rgb(0, 0, 0),
            align: TextAlign {},
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum NodeTexture {
    Text,
    Image,
}

impl NodeTexture {
    pub fn all() -> Vec<NodeTexture> {
        vec![NodeTexture::Text, NodeTexture::Image]
    }
}

#[derive(Clone)]
struct VisualState {
    textures: HashMap<NodeTexture, TextureID>,
    texture_req: HashSet<NodeTexture>,
    transform: Transform,
    style: Style,
    text: Text,
}

impl VisualState {
    fn new() -> Self {
        Self {
            textures: HashMap::new(),
            texture_req: HashSet::new(),
            transform: Transform::new(),
            style: Style::new(),
            text: Text::new(),
        }
    }

    fn add_texture_requirement(&mut self, req: NodeTexture) {
        self.texture_req.insert(req);
    }
}

#[derive(Clone, Copy)]
enum CurrentVisualState {
    Normal,
    Hover,
    Active,
}

#[derive(Clone)]
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

    fn current_state(&self) -> &VisualState {
        match self.current {
            CurrentVisualState::Normal => &self.normal,
            CurrentVisualState::Hover => &self.hover,
            CurrentVisualState::Active => &self.active,
        }
    }

    fn current_state_mut(&mut self) -> &mut VisualState {
        match self.current {
            CurrentVisualState::Normal => &mut self.normal,
            CurrentVisualState::Hover => &mut self.hover,
            CurrentVisualState::Active => &mut self.active,
        }
    }

    fn fusion_state<T>(&mut self, origin: StyleActionOrigin, mut callback: T)
    where
        T: FnMut(&mut VisualState),
    {
        match origin {
            StyleActionOrigin::Code => {
                self.set_normal();
            }
            StyleActionOrigin::Hover => {
                self.set_hover();
            }
            StyleActionOrigin::Active => {
                self.set_active();
            }
        }
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

    /* Texture Management */

    pub fn get_texture(&self, req: NodeTexture) -> Option<TextureID> {
        self.current_state().textures.get(&req).copied()
    }

    pub fn get_all_textures(&self) -> Vec<TextureID> {
        let mut textures: Vec<TextureID> = Vec::new();
        textures.extend(self.normal.textures.values());
        textures.extend(self.hover.textures.values());
        textures.extend(self.active.textures.values());
        textures
    }

    pub fn texture_required(&self, req: NodeTexture) -> bool {
        self.current_state().texture_req.contains(&req)
    }

    pub fn set_texture(&mut self, id: TextureID, req: NodeTexture) {
        self.current_state_mut().textures.insert(req, id);
        self.current_state_mut().texture_req.remove(&req);
    }

    /* Getter */

    pub fn get_bg_color(&self) -> Color {
        self.current_state().style.bg_color
    }

    pub fn get_border_radius(&self) -> u32 {
        self.current_state().style.border_radius
    }

    pub fn get_current_timer(&self) -> Option<Duration> {
        self.current_state().style.timer
    }

    pub fn get_current_timer_start(&self) -> Option<Instant> {
        self.current_state().style.timer_start
    }

    pub fn get_size(&self) -> Size {
        self.current_state().style.size
    }

    pub fn get_relative_position(&self) -> Position {
        self.current_state().transform.position
    }

    pub fn is_position_absolute(&self) -> bool {
        self.current_state().transform.position_absolute
    }

    pub fn get_rotation(&self) -> f32 {
        self.current_state().transform.rotation
    }

    pub fn get_scale(&self) -> Scale {
        self.current_state().transform.scale
    }

    pub fn get_layer(&self) -> Option<usize> {
        self.current_state().transform.layer
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

    pub fn get_content(&self) -> String {
        if let Some(content) = &self.current_state().text.content {
            content.clone()
        } else {
            "".to_string()
        }
    }

    pub fn get_text_align(&self) -> TextAlign {
        self.current_state().text.align
    }

    pub fn get_font_size(&self) -> u16 {
        self.current_state().text.font_size
    }

    pub fn get_font(&self) -> Option<FontID> {
        self.current_state().text.font
    }

    pub fn get_font_style(&self) -> FontStyle {
        self.current_state().text.font_style
    }

    pub fn get_text_color(&self) -> Color {
        self.current_state().text.color
    }

    /* Setter */

    pub fn set_bg_color(&mut self, color: Color, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.style.bg_color = color;
        });
    }

    pub fn set_border_radius(&mut self, radius: u32, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.style.border_radius = radius;
        });
    }

    pub fn set_timer(&mut self, ms: u64) {
        self.current_state_mut().style.timer = Some(Duration::from_millis(ms));
        self.current_state_mut().style.timer_start = Some(Instant::now());
    }

    pub fn set_size(&mut self, width: u32, height: u32, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.style.size = Size { width, height };
        });
    }

    pub fn set_position(&mut self, position: Position, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.transform.position = position;
        });
    }

    pub fn set_position_absolute(&mut self, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.transform.position_absolute = true;
        });
    }

    pub fn set_position_relative(&mut self, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.transform.position_absolute = false;
        });
    }

    pub fn set_rotation(&mut self, rotation: f32, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.transform.rotation = rotation;
        });
    }

    pub fn set_scale(&mut self, x: f32, y: f32, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.transform.scale = Scale { x, y };
        });
    }

    pub fn set_layer(&mut self, layer: usize, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.transform.layer = Some(layer);
        });
    }

    pub(crate) fn clear_current_timer(&mut self) {
        self.current_state_mut().style.timer = None;
        self.current_state_mut().style.timer_start = None;
    }

    pub(crate) fn clear_normal_timer(&mut self) {
        self.normal.style.timer = None;
        self.normal.style.timer_start = None;
    }

    pub(crate) fn clear_hover_timer(&mut self) {
        self.hover.style.timer = None;
        self.hover.style.timer_start = None;
    }

    pub(crate) fn clear_active_timer(&mut self) {
        self.active.style.timer = None;
        self.active.style.timer_start = None;
    }

    pub fn set_content(&mut self, content: String, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.text.content = Some(content.clone());
            state.add_texture_requirement(NodeTexture::Text);
        });
    }

    pub fn set_text_align(&mut self, align: TextAlign, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.text.align = align;
            state.add_texture_requirement(NodeTexture::Text);
        });
    }

    pub fn set_font_size(&mut self, size: u16, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.text.font_size = size;
            state.add_texture_requirement(NodeTexture::Text);
        });
    }

    pub fn set_font(&mut self, font: FontID, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.text.font = Some(font);
            state.add_texture_requirement(NodeTexture::Text);
        });
    }

    pub fn set_font_style(&mut self, style: FontStyle, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.text.font_style = style;
            state.add_texture_requirement(NodeTexture::Text);
        });
    }

    pub fn set_text_color(&mut self, color: Color, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.text.color = color;
            state.add_texture_requirement(NodeTexture::Text);
        });
    }

    pub fn set_bg_image(&mut self, image: TextureID, origin: StyleActionOrigin) {
        self.fusion_state(origin, |state| {
            state.style.bg_image = Some(image);
            state.add_texture_requirement(NodeTexture::Image);
        });
    }

    /* State Change */

    pub(crate) fn set_normal(&mut self) {
        self.current = CurrentVisualState::Normal;
    }

    pub(crate) fn set_hover(&mut self) {
        self.current = CurrentVisualState::Hover;
    }

    pub(crate) fn set_active(&mut self) {
        self.current = CurrentVisualState::Active;
    }
}
