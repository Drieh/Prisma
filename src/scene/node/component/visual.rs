use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::{
    resources::{ResourceID, TextureID},
    util::{
        Background, Border, DisplayLayout, DisplayPosition, Margin, Padding, Size, StyleCategory,
        StyleTextureCategory, Text, Transform,
    },
};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum StyleTextureType {
    Background,
    Text,
}

#[derive(Clone, PartialEq, Debug)]
pub enum StyleTextureSeed {
    Text(Text),
    Background(Background),
}

#[derive(Clone, PartialEq, Debug)]
pub struct CachedTexture {
    seed: StyleTextureSeed,
    texture_id: TextureID,
}

impl CachedTexture {
    pub fn seed<T: StyleTextureCategory>(&self) -> T {
        T::from_seed(&self.seed)
    }

    pub fn id(&self) -> TextureID {
        self.texture_id
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct TextureCache {
    pub textures: HashMap<StyleTextureType, CachedTexture>,
}

impl TextureCache {
    fn insert(
        &mut self,
        texture_type: StyleTextureType,
        texture_id: TextureID,
        seed: StyleTextureSeed,
    ) {
        self.textures
            .insert(texture_type, CachedTexture { seed, texture_id });
    }

    fn get(&self, texture_type: StyleTextureType) -> Option<CachedTexture> {
        self.textures.get(&texture_type).cloned()
    }

    fn values(&self) -> Vec<CachedTexture> {
        self.textures.values().cloned().collect()
    }
}

impl Default for TextureCache {
    fn default() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ResolvedVisualState {
    pub display: DisplayLayout,
    pub position: DisplayPosition,

    pub transform: Transform,
    pub background: Background,
    pub border: Border,
    pub padding: Padding,
    pub margin: Margin,
    pub text: Text,

    pub size: Size,
    pub inline_size: Size,
    pub timer: Option<Duration>,
    pub timer_start: Option<Instant>,
}

impl Default for ResolvedVisualState {
    fn default() -> Self {
        Self {
            display: DisplayLayout::Inline,
            position: DisplayPosition::Static,
            transform: Transform::default(),
            background: Background::default(),
            border: Border::default(),
            padding: Padding::default(),
            margin: Margin::default(),
            text: Text::default(),
            size: Size {
                width: 0,
                height: 0,
            },
            inline_size: Size {
                width: 0,
                height: 0,
            },
            timer: None,
            timer_start: None,
        }
    }
}

impl ResolvedVisualState {
    fn merge(mut self, layer: &UnresolvedVisualState) -> Self {
        let UnresolvedVisualState {
            display,
            position,
            transform,
            background,
            border,
            padding,
            margin,
            text,
            size,
            inline_size,
            timer,
            timer_start,
        } = layer;

        if let Some(v) = display {
            self.display = *v;
        }
        if let Some(v) = position {
            self.position = *v;
        }
        if let Some(v) = size {
            self.size = *v;
        }
        if let Some(v) = inline_size {
            self.inline_size = *v;
        }

        self.transform.merge(transform);
        self.background.merge(background);
        self.border.merge(border);
        self.padding.merge(padding);
        self.margin.merge(margin);
        self.text.merge(text);

        self.timer = *timer;
        self.timer_start = *timer_start;

        self
    }

    fn normal(normal: &UnresolvedVisualState) -> Self {
        Self::default().merge(normal)
    }

    fn hover(normal: &UnresolvedVisualState, hover: &UnresolvedVisualState) -> Self {
        Self::default().merge(normal).merge(hover)
    }

    fn active(
        normal: &UnresolvedVisualState,
        hover: &UnresolvedVisualState,
        active: &UnresolvedVisualState,
    ) -> Self {
        Self::default().merge(normal).merge(hover).merge(active)
    }
}

#[derive(Clone)]
struct UnresolvedVisualState {
    display: Option<DisplayLayout>,
    position: Option<DisplayPosition>,

    background: Background,
    border: Border,
    margin: Margin,
    padding: Padding,
    text: Text,
    transform: Transform,

    size: Option<Size>,
    inline_size: Option<Size>,
    timer: Option<Duration>,
    timer_start: Option<Instant>,
}

impl UnresolvedVisualState {
    fn empty() -> Self {
        Self {
            display: None,
            position: None,

            transform: Transform::empty(),
            background: Background::empty(),
            border: Border::empty(),
            padding: Padding::empty(),
            margin: Margin::empty(),
            text: Text::empty(),

            size: None,
            inline_size: None,
            timer: None,
            timer_start: None,
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum StyleVisualState {
    Normal,
    Hover,
    Active,
}

#[derive(Clone)]
pub struct NodeVisual {
    normal: UnresolvedVisualState,
    hover: UnresolvedVisualState,
    active: UnresolvedVisualState,
    current: StyleVisualState,

    pub(crate) texture_cache: HashMap<StyleVisualState, TextureCache>,
    pub(crate) viewport: Size,
}

impl NodeVisual {
    pub fn new() -> Self {
        Self {
            normal: UnresolvedVisualState::empty(),
            hover: UnresolvedVisualState::empty(),
            active: UnresolvedVisualState::empty(),
            current: StyleVisualState::Normal,
            texture_cache: HashMap::new(),
            viewport: Size {
                width: 0,
                height: 0,
            },
        }
    }

    pub fn normal(&self) -> ResolvedVisualState {
        ResolvedVisualState::normal(&self.normal)
    }

    pub fn hover(&self) -> ResolvedVisualState {
        ResolvedVisualState::hover(&self.normal, &self.hover)
    }

    pub fn active(&self) -> ResolvedVisualState {
        ResolvedVisualState::active(&self.normal, &self.hover, &self.active)
    }

    pub fn current_resolved_visual_state(&self) -> ResolvedVisualState {
        match self.current {
            StyleVisualState::Normal => self.normal(),
            StyleVisualState::Hover => self.hover(),
            StyleVisualState::Active => self.active(),
        }
    }

    fn current_unresolved_visual_state(&mut self) -> &mut UnresolvedVisualState {
        match self.current {
            StyleVisualState::Normal => &mut self.normal,
            StyleVisualState::Hover => &mut self.hover,
            StyleVisualState::Active => &mut self.active,
        }
    }

    fn modify_unresolved_visual_state<T>(&mut self, origin: StyleVisualState, mut callback: T)
    where
        T: FnMut(&mut UnresolvedVisualState),
    {
        match origin {
            StyleVisualState::Normal => callback(&mut self.normal),
            StyleVisualState::Hover => callback(&mut self.hover),
            StyleVisualState::Active => callback(&mut self.active),
        }
    }

    /* Textures */
    pub(crate) fn get_all_textures(&self) -> Vec<TextureID> {
        let mut textures = Vec::new();
        for cache in self.texture_cache.values() {
            textures.extend(cache.values().iter().filter_map(|texture| {
                if texture.id() == TextureID::none() {
                    None
                } else {
                    Some(texture.id())
                }
            }));
        }
        textures
    }

    pub(crate) fn set_texture_cache(
        &mut self,
        texture_type: StyleTextureType,
        texture: TextureID,
        state: StyleVisualState,
    ) {
        let seed = match texture_type {
            StyleTextureType::Background => StyleTextureSeed::Background(self.get_background()),
            StyleTextureType::Text => StyleTextureSeed::Text(self.get_text()),
        };
        self.texture_cache
            .entry(state)
            .or_default()
            .insert(texture_type, texture, seed);
    }

    pub fn get_texture_cache(&mut self, texture_type: StyleTextureType) -> Option<CachedTexture> {
        let asd = self
            .texture_cache
            .entry(self.current)
            .or_default()
            .get(texture_type);
        asd
    }

    /* Getter */

    pub fn current_visual_state(&self) -> StyleVisualState {
        self.current
    }

    pub fn get_display(&self) -> DisplayLayout {
        self.current_resolved_visual_state().display
    }

    pub fn get_position(&self) -> DisplayPosition {
        self.current_resolved_visual_state().position
    }

    pub fn get_transform(&self) -> Transform {
        self.current_resolved_visual_state().transform
    }

    pub fn get_background(&self) -> Background {
        self.current_resolved_visual_state().background
    }

    pub fn get_padding(&self) -> Padding {
        self.current_resolved_visual_state().padding
    }

    pub fn get_border(&self) -> Border {
        self.current_resolved_visual_state().border
    }

    pub fn get_margin(&self) -> Margin {
        self.current_resolved_visual_state().margin
    }

    pub fn get_text(&self) -> Text {
        self.current_resolved_visual_state().text.clone()
    }

    pub fn get_size(&self) -> Size {
        self.current_resolved_visual_state().size
    }

    pub fn get_inline_size(&self) -> Size {
        self.current_resolved_visual_state().inline_size
    }

    pub fn get_current_timer(&self) -> Option<Duration> {
        self.current_resolved_visual_state().timer
    }

    pub fn get_current_timer_start(&self) -> Option<Instant> {
        self.current_resolved_visual_state().timer_start
    }

    /* Setter */

    pub fn set_display(&mut self, display: DisplayLayout, origin: StyleVisualState) {
        self.modify_unresolved_visual_state(origin, |state| {
            state.display = Some(display);
        });
    }

    pub fn set_position(&mut self, position: DisplayPosition, origin: StyleVisualState) {
        self.modify_unresolved_visual_state(origin, |state| {
            state.position = Some(position);
        });
    }

    pub fn set_transform(&mut self, transform: Transform, origin: StyleVisualState) {
        self.modify_unresolved_visual_state(origin, |state| {
            state.transform = transform.clone();
        });
    }

    pub fn set_background(&mut self, background: Background, origin: StyleVisualState) {
        self.modify_unresolved_visual_state(origin, |state| {
            state.background = background.clone();
        });
    }

    pub fn set_border(&mut self, border: Border, origin: StyleVisualState) {
        self.modify_unresolved_visual_state(origin, |state| {
            state.border = border.clone();
        });
    }

    pub fn set_padding(&mut self, padding: Padding, origin: StyleVisualState) {
        self.modify_unresolved_visual_state(origin, |state| {
            state.padding = padding.clone();
        });
    }

    pub fn set_margin(&mut self, margin: Margin, origin: StyleVisualState) {
        self.modify_unresolved_visual_state(origin, |state| {
            state.margin = margin.clone();
        });
    }

    pub fn set_text(&mut self, text: Text, origin: StyleVisualState) {
        self.modify_unresolved_visual_state(origin, move |state| {
            state.text = text.clone();
        });
    }

    pub fn set_size(&mut self, size: Size, origin: StyleVisualState) {
        self.modify_unresolved_visual_state(origin, |state| {
            state.size = Some(size);
        });
    }

    pub fn set_inline_size(&mut self, size: Size, origin: StyleVisualState) {
        self.modify_unresolved_visual_state(origin, |state| {
            state.inline_size = Some(size);
        });
    }

    pub fn wait(&mut self, ms: u64) {
        self.current_unresolved_visual_state().timer = Some(Duration::from_millis(ms));
        self.current_unresolved_visual_state().timer_start = Some(Instant::now());
    }

    /* State Change */

    pub(crate) fn set_normal(&mut self) {
        self.current = StyleVisualState::Normal;
    }

    pub(crate) fn set_hover(&mut self) {
        self.current = StyleVisualState::Hover;
    }

    pub(crate) fn set_active(&mut self) {
        self.current = StyleVisualState::Active;
    }

    pub(crate) fn clear_current_state_timer(&mut self) {
        self.current_unresolved_visual_state().timer = None;
        self.current_unresolved_visual_state().timer_start = None;
    }

    pub(crate) fn clear_normal_timer(&mut self) {
        self.normal.timer = None;
        self.normal.timer_start = None;
    }

    pub(crate) fn clear_hover_timer(&mut self) {
        self.hover.timer = None;
        self.hover.timer_start = None;
    }

    pub(crate) fn clear_active_timer(&mut self) {
        self.active.timer = None;
        self.active.timer_start = None;
    }
}
