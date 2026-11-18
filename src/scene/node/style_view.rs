use crate::{
    node::{
        StyleAction,
        component::{NodeQueue, StyleVisualState},
    },
    util::{
        Background, Border, DisplayLayout, DisplayPosition, Margin, Padding, Size, Text, Transform,
    },
};

pub type StyleCallback = Box<dyn FnMut(&mut StyleView<'_>) + 'static>;

/// This is the base for building complex styles.
///
/// # Behavior
///
/// All operations of this implementation are deferred.
pub struct StyleView<'a> {
    queue: &'a mut NodeQueue,
    origin: StyleVisualState,
}
impl<'a> StyleView<'a> {
    pub(crate) fn new(queue: &'a mut NodeQueue, origin: StyleVisualState) -> Self {
        Self { queue, origin }
    }

    fn push_action(&mut self, action: StyleAction) {
        self.queue.push_back::<StyleAction>(action);
    }

    /// Sets the node's display.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn display(&mut self, display: DisplayLayout) -> &mut Self {
        self.push_action(StyleAction::Display {
            origin: self.origin,
            display,
        });
        self
    }

    pub fn size(&mut self, size: Size) -> &mut Self {
        self.push_action(StyleAction::Size {
            origin: self.origin,
            size,
        });
        self
    }

    /// Sets the node's position behavior.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn position(&mut self, position: DisplayPosition) -> &mut Self {
        self.push_action(StyleAction::Position {
            origin: self.origin,
            position,
        });
        self
    }

    pub fn transform<T>(&mut self, callback: T) -> &mut Self
    where
        T: FnMut(&mut Transform) + 'static,
    {
        self.push_action(StyleAction::Transform {
            origin: self.origin,
            callback: Box::new(callback),
        });
        self
    }

    pub fn background<T>(&mut self, callback: T) -> &mut Self
    where
        T: FnMut(&mut Background) + 'static,
    {
        self.push_action(StyleAction::Background {
            origin: self.origin,
            callback: Box::new(callback),
        });
        self
    }

    /// Sets the node's border as declared.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn border<T>(&mut self, callback: T) -> &mut Self
    where
        T: FnMut(&mut Border) + 'static,
    {
        self.push_action(StyleAction::Border {
            origin: self.origin,
            callback: Box::new(callback),
        });
        self
    }

    /// Sets the node's padding as declared.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn padding<T>(&mut self, callback: T) -> &mut Self
    where
        T: FnMut(&mut Padding) + 'static,
    {
        self.push_action(StyleAction::Padding {
            origin: self.origin,
            callback: Box::new(callback),
        });
        self
    }

    /// Sets the node's text as declared.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn text<T>(&mut self, callback: T) -> &mut Self
    where
        T: FnMut(&mut Text) + 'static,
    {
        self.push_action(StyleAction::Text {
            origin: self.origin,
            callback: Box::new(callback),
        });
        self
    }

    /// Sets the node's padding as declared.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn margin<T>(&mut self, callback: T) -> &mut Self
    where
        T: FnMut(&mut Margin) + 'static,
    {
        self.push_action(StyleAction::Margin {
            origin: self.origin,
            callback: Box::new(callback),
        });
        self
    }

    /// Adds a timer to the node's style queue, delaying the execution of its actions.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn wait(&mut self, ms: u64) -> &mut Self {
        self.push_action(StyleAction::Wait { ms });
        self
    }
}
