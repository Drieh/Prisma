use crate::{
    node::{StyleAction, StyleActionOrigin, component::NodeQueue},
    resources::{
        FontID, TextureID,
        font::{FontStyle, TextAlign},
    },
    util::{Color, Position},
};

pub type StyleCallback = Box<dyn FnMut(&mut StyleView<'_>) + 'static>;

/// This is the base for building complex styles.
///
/// # Behavior
///
/// All operations of this implementation are deferred.
pub struct StyleView<'a> {
    queue: &'a mut NodeQueue,
    origin: StyleActionOrigin,
}
impl<'a> StyleView<'a> {
    pub(crate) fn new(queue: &'a mut NodeQueue, origin: StyleActionOrigin) -> Self {
        Self { queue, origin }
    }

    fn push_style(&mut self, action: StyleAction) {
        self.queue.push_back::<StyleAction>(action);
    }

    /* Basic */

    /// Sets the node's position to the given `x` and `y`.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn position(&mut self, x: i32, y: i32) -> &mut Self {
        self.push_style(StyleAction::Position {
            origin: self.origin,
            position: Some(Position { x, y }),
            absolute: None,
        });
        self
    }

    /// Sets the node's position as an absolute position relative to the scene.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn position_absolute(&mut self) -> &mut Self {
        self.push_style(StyleAction::Position {
            origin: self.origin,
            position: None,
            absolute: Some(true),
        });
        self
    }

    /// Sets the node's position relative to its parent or the scene if it has no parent.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn position_relative(&mut self) -> &mut Self {
        self.push_style(StyleAction::Position {
            origin: self.origin,
            position: None,
            absolute: Some(false),
        });
        self
    }

    /// Sets the node's scale.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn scale(&mut self, x: f32, y: f32) -> &mut Self {
        self.push_style(StyleAction::Scale {
            origin: self.origin,
            x,
            y,
        });
        self
    }

    /// Sets the node's size.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn size(&mut self, width: u32, height: u32) -> &mut Self {
        self.push_style(StyleAction::Size {
            origin: self.origin,
            width,
            height,
        });
        self
    }

    /// Sets the node's background color.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn bg_color(&mut self, color: Color) -> &mut Self {
        self.push_style(StyleAction::BGColor {
            origin: self.origin,
            color,
        });
        self
    }

    /// Sets the node's render layer.
    ///
    /// Higher layers are rendered on top of lower layers.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn layer(&mut self, layer: usize) -> &mut Self {
        self.push_style(StyleAction::Layer {
            origin: self.origin,
            layer,
        });
        self
    }

    /// Sets the node's border radius.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn border_radius(&mut self, radius: u32) -> &mut Self {
        self.push_style(StyleAction::BorderRadius {
            origin: self.origin,
            radius,
        });
        self
    }

    /// Adds a timer to the node's style queue, delaying the execution of its actions.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn wait(&mut self, ms: u64) -> &mut Self {
        self.push_style(StyleAction::Wait { ms });
        self
    }

    /* Text */

    /// Sets the node's text.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn content(&mut self, content: String) -> &mut Self {
        self.push_style(StyleAction::TextContent {
            origin: self.origin,
            content,
        });
        self
    }

    /// Sets the node's text color.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn text_color(&mut self, color: Color) -> &mut Self {
        self.push_style(StyleAction::TextColor {
            origin: self.origin,
            color,
        });
        self
    }

    /// Sets the node's text align.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn text_align(&mut self, align: TextAlign) -> &mut Self {
        self.push_style(StyleAction::TextAlign {
            origin: self.origin,
            align,
        });
        self
    }

    /// Sets the node's text font.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn font_family(&mut self, font: FontID) -> &mut Self {
        self.push_style(StyleAction::Font {
            origin: self.origin,
            font,
        });
        self
    }

    /// Sets the node's text font size.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn font_size(&mut self, size: u16) -> &mut Self {
        self.push_style(StyleAction::FontSize {
            origin: self.origin,
            size,
        });
        self
    }

    /// Sets the node's text font style.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn font_style(&mut self, style: FontStyle) -> &mut Self {
        self.push_style(StyleAction::FontStyle {
            origin: self.origin,
            style,
        });
        self
    }

    /* Images */

    /// Sets the node's background image.
    ///
    /// Note: the image must be registered in the prisma app variable.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn bg_image(&mut self, image: TextureID) -> &mut Self {
        self.push_style(StyleAction::BGImage {
            origin: self.origin,
            image,
        });
        self
    }
}
