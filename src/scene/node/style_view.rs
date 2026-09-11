use crate::{
    node::{ActionOrigin, component::NodeQueue, node_action::StyleAction},
    util::{Color, Position},
};

pub type StyleCallback = dyn FnMut(&mut StyleView<'_>) + 'static;

pub struct StyleView<'a> {
    queue: &'a mut NodeQueue,
    origin: ActionOrigin,
}
impl<'a> StyleView<'a> {
    pub(crate) fn new(queue: &'a mut NodeQueue, origin: ActionOrigin) -> Self {
        Self { queue, origin }
    }

    fn push_action(&mut self, action: StyleAction) {
        self.queue.push_back::<StyleAction>(action, self.origin);
    }

    /// Sets the node's position to the given `x` and `y`.
    ///
    /// Returns a mutable reference to [`Self`]
    pub fn position(&mut self, x: i32, y: i32) -> &mut Self {
        self.push_action(StyleAction::Position {
            position: Some(Position { x, y }),
            absolute: None,
        });
        self
    }

    /// Sets the node's position as an absolute position relative to the scene.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn position_absolute(&mut self) -> &mut Self {
        self.push_action(StyleAction::Position {
            position: None,
            absolute: Some(true),
        });
        self
    }

    /// Sets the node's position relative to its parent or the scene if it has no parent.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn position_relative(&mut self) -> &mut Self {
        self.push_action(StyleAction::Position {
            position: None,
            absolute: Some(false),
        });
        self
    }

    /// Sets the node's scale.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn scale(&mut self, x: f32, y: f32) -> &mut Self {
        self.push_action(StyleAction::Scale { x, y });
        self
    }

    /// Sets the node's size.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn size(&mut self, width: u32, height: u32) -> &mut Self {
        self.push_action(StyleAction::Size { width, height });
        self
    }

    /// Sets the node's background color.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn bg_color(&mut self, color: Color) -> &mut Self {
        self.push_action(StyleAction::BGColor { color });
        self
    }

    /// Sets the node's render layer.
    ///
    /// Higher layers are rendered on top of lower layers.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn layer(&mut self, layer: usize) -> &mut Self {
        self.push_action(StyleAction::Layer { layer });
        self
    }

    /// Sets the node's border radius.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn border_radius(&mut self, radius: u32) -> &mut Self {
        self.push_action(StyleAction::BorderRadius { radius });
        self
    }

    /// Adds a timer to the node's [`NodeAction`] queue, delaying the execution of its actions.
    pub(crate) fn wait(&mut self, ms: u64) -> &mut Self {
        self.push_action(StyleAction::Wait { ms });
        self
    }
}
