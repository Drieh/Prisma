use std::time::Duration;

use crate::{
    node::{Action, ActionQueue, NodeAction},
    util::{Color, Position},
};

pub type StyleCallback = dyn FnMut(&mut StyleView<'_>) + 'static;
pub struct StyleView<'a> {
    action_queue: &'a mut ActionQueue,
    persistent: bool,
}
impl<'a> StyleView<'a> {
    pub(crate) fn new(action_queue: &'a mut ActionQueue, persistent: bool) -> Self {
        Self {
            action_queue,
            persistent,
        }
    }

    /// Sets the node's position to the given `x` and `y`.
    ///
    /// Returns a mutable reference to [`Self`]
    pub fn position(&mut self, x: i32, y: i32) {
        let x = x as f32;
        let y = y as f32;

        self.action_queue.push_back(NodeAction::new(
            Action::Position {
                position: Some(Position { x, y }),
                absolute: None,
            },
            self.persistent,
        ));
    }

    /// Sets the node's position as an absolute position relative to the scene.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn position_absolute(&mut self) -> &mut Self {
        self.action_queue.push_back(NodeAction::new(
            Action::Position {
                position: None,
                absolute: Some(true),
            },
            self.persistent,
        ));
        self
    }

    /// Sets the node's position relative to its parent or the scene if it has no parent.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn position_relative(&mut self) -> &mut Self {
        self.action_queue.push_back(NodeAction::new(
            Action::Position {
                position: None,
                absolute: Some(false),
            },
            self.persistent,
        ));
        self
    }

    /// Sets the node's scale.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn scale(&mut self, x: f32, y: f32) -> &mut Self {
        self.action_queue
            .push_back(NodeAction::new(Action::Scale { x, y }, self.persistent));
        self
    }

    /// Sets the node's size.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn size(&mut self, width: u32, height: u32) -> &mut Self {
        self.action_queue.push_back(NodeAction::new(
            Action::Size { width, height },
            self.persistent,
        ));
        self
    }

    /// Sets the node's background color.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn bg_color(&mut self, color: Color) -> &mut Self {
        self.action_queue
            .push_back(NodeAction::new(Action::BGColor { color }, self.persistent));
        self
    }

    /// Sets the node's render layer.
    ///
    /// Higher layers are rendered on top of lower layers.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn layer(&mut self, layer: usize) -> &mut Self {
        self.action_queue
            .push_back(NodeAction::new(Action::Layer { layer }, self.persistent));
        self
    }

    /// Sets the node's border radius.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn border_radius(&mut self, radius: u32) -> &mut Self {
        self.action_queue.push_back(NodeAction::new(
            Action::BorderRadius { radius },
            self.persistent,
        ));
        self
    }

    /// Adds a timer to the node's [`NodeAction`] queue, delaying the execution of its actions.
    pub(crate) fn wait(&mut self, ms: u64) -> &mut Self {
        self.action_queue.push_back(NodeAction::new(
            Action::Wait {
                duration: Duration::from_millis(ms),
            },
            false,
        ));
        self
    }
}
