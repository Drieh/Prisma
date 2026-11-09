use std::time::Instant;

use crate::error::PrismaError;
use crate::event::EventCallback;
use crate::event::EventCallbackID;
use crate::event::EventContext;
use crate::event::EventData;
use crate::event::EventManager;
use crate::node::Action;
use crate::node::EventListenerAction;
use crate::node::NodeTreeAction;
use crate::node::StyleAction;
use crate::node::StyleActionOrigin;
use crate::node::StyleView;
use crate::node::component::NodeTexture;
use crate::render::Renderer;
use crate::resources::manager::ResourceManager;
use crate::scene::NodeID;
use crate::scene::NodeView;
use crate::scene::handler::CloseHandler;
use crate::scene::handler::PendingNodesHandler;
use crate::scene::handler::TextureHandler;
use crate::scene::handler::TextureRequirement;
use crate::scene::storage::{NodeStorage, StorageHandler};
use crate::util::Color;

/// Represents a UI scene displayed inside an window.
///
/// A scene owns all nodes and processes events within its window.
pub struct Scene {
    pub color: Color,
    nodes: NodeStorage,
    event_manager: EventManager,

    pending_nodes_handler: PendingNodesHandler,
    close_handler: CloseHandler,
    texture_handler: TextureHandler,
}
impl Scene {
    pub fn new() -> Self {
        Self {
            color: Color::rgb(255, 255, 255),
            nodes: NodeStorage::new(),
            event_manager: EventManager::new(),
            pending_nodes_handler: PendingNodesHandler::new(),
            close_handler: CloseHandler::new(),
            texture_handler: TextureHandler::new(),
        }
    }

    pub fn storage(&mut self) -> StorageHandler<'_> {
        self.nodes.storage()
    }

    /// Creates a new node and returns a mutable [`NodeView`] to it.
    ///
    /// The returned view can be used to configure the node before or after Prisma is running.
    pub fn new_node(&mut self) -> NodeView<'_> {
        let new_node = self.nodes.new_node();
        self.pending_nodes_handler.push_created(new_node.get_id());
        new_node
    }

    /// Returns `true` if the scene contains a node with the given ID.
    pub fn contains(&mut self, id: NodeID) -> bool {
        self.nodes.exists(id)
    }

    /// Returns a list of all available nodes.
    pub fn get_nodes(&self) -> Vec<NodeID> {
        self.nodes.get_nodes_id()
    }

    /// Returns a [`NodeView`] of the given [`NodeID`].
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeNotFound`] if there is no node for the given [`NodeID`].
    pub fn get_node(&mut self, id: NodeID) -> Result<NodeView<'_>, PrismaError> {
        self.nodes.get_node_view(id)
    }

    /// Sets the background color of the scene.
    pub fn bg_color(&mut self, color: Color) {
        self.color = Color { a: 255, ..color };
    }

    pub fn get_bg_color(&self) -> Color {
        self.color
    }

    /// Returns `true` if window is about to close.
    pub fn is_quitting(&self) -> bool {
        self.close_handler.is_quitting()
    }

    pub(crate) fn process_textures(
        &mut self,
        resources: &mut ResourceManager,
        renderer: &Renderer,
    ) -> Result<(), PrismaError> {
        let StorageHandler { mut visual, .. } = self.nodes.storage();

        for TextureRequirement { node, req } in self.texture_handler.take_queue() {
            let node_visual = visual.get_unchecked_mut(node);
            if let Some(texture) = node_visual.get_texture(req) {
                println!("removed user {node:?} from {texture:?}");
                resources.remove_user(texture, node)?;
            }
            match req {
                NodeTexture::Text => {
                    let texture = renderer.create_text_texture(node_visual, resources)?;
                    let texture_id = resources.register_internal(texture)?;

                    println!("added user {node:?} for {texture_id:?}");
                    resources.register_user(texture_id, node)?;
                    node_visual.set_texture(texture_id, req);
                }
                NodeTexture::Image => todo!(),
            }
        }
        Ok(())
    }

    pub fn destroy_node(&mut self, id: NodeID) -> Result<(), PrismaError> {
        self.nodes.destroy_node(id)
    }

    /* Events */

    /// Registers a callback for the given event type.
    ///
    /// # Arguments
    ///
    /// * `event_type` - The event to listen for.
    /// * `callback` - The function to invoke when the event is dispatched.
    pub fn on_event<T>(&mut self, mut callback: impl FnMut(&mut EventContext, T) + 'static)
    where
        T: EventData,
    {
        let real_callback = move |ctx: &mut EventContext| {
            let event = ctx.expect_event::<T>().unwrap();
            callback(ctx, event);
        };
        self.event_manager
            .scene_on_event(T::TYPE, EventCallback::new(real_callback));
    }

    /// Removes the callback identified by the given [`CallbackID`].
    pub fn off_event(&mut self, target: EventCallbackID) {
        self.event_manager.off_event(target);
    }

    pub(crate) fn create_lifecycle_events(&mut self) {
        let pending_created_nodes = self.pending_nodes_handler.take_created();
        let pending_destroyed_nodes = self.pending_nodes_handler.get_destroyed();

        self.event_manager
            .manage_lifecycle_events(&pending_created_nodes, pending_destroyed_nodes);
    }

    pub(crate) fn create_user_event(&mut self, event: &sdl3::event::Event) {
        self.event_manager.manage_sdl_event(event);
    }

    pub(crate) fn dispatch_events(&mut self) {
        let mut context = EventContext::new(&mut self.nodes);

        self.event_manager.dispatch(&mut context);
        self.event_manager
            .process_scene_callback_actions(&mut context);
        self.pending_nodes_handler.extend_pending(&mut context);

        self.close_handler
            .handle_close(&mut context, &mut self.event_manager);
    }

    pub(crate) fn process_scene(
        &mut self,
        resources: &mut ResourceManager,
        renderer: &Renderer,
    ) -> Result<(), PrismaError> {
        for id in self.nodes.get_nodes_id() {
            self.process_node_tree_actions(id);
            self.process_event_listener_actions(id);
            self.process_node_style_actions(id);
            self.process_node_destruction_queue(id);
        }
        self.process_textures(resources, renderer)?;

        for id in self.pending_nodes_handler.take_destroyed() {
            let StorageHandler { visual, .. } = self.nodes.storage();

            let node_visual = visual.get_unchecked(id);
            for texture in node_visual.get_all_textures() {
                resources.remove_user(texture, id)?;
            }
            self.event_manager.clear_node_listeners(id);
            self.destroy_node(id).expect("Failded to destroy node");
        }
        Ok(())
    }

    fn process_event_listener_actions(&mut self, id: NodeID) {
        let StorageHandler { mut queue, .. } = self.nodes.storage();
        while let Some(listener_action) = queue
            .get_unchecked_mut(id)
            .pop_front::<EventListenerAction>()
        {
            match listener_action {
                EventListenerAction::Add {
                    event_type,
                    callback,
                } => {
                    self.event_manager.node_on_event(id, event_type, callback);
                }
                EventListenerAction::Remove { target } => {
                    self.event_manager.off_event(target);
                }
            }
        }
    }

    fn process_node_style_actions(&mut self, id: NodeID) {
        let StorageHandler {
            mut state,
            mut queue,
            mut visual,
            ..
        } = self.nodes.storage();
        let node_state = state.get_unchecked_mut(id);
        let node_visual = visual.get_unchecked_mut(id);
        let node_queue = queue.get_unchecked_mut(id);

        let is_hovered = self.event_manager.is_node_hovered(id);
        let is_active = self.event_manager.is_node_active(id);
        let was_hovered = node_state.is_hovered;
        let was_active = node_state.is_active;

        if is_hovered
            && !was_hovered
            && !is_active
            && let Some(style_callback) = &mut node_state.style_on_hover
        {
            let mut style = StyleView::new(node_queue, StyleActionOrigin::Hover);
            style_callback(&mut style);
        }

        if is_active
            && !was_active
            && let Some(style_callback) = &mut node_state.style_on_active
        {
            let mut style = StyleView::new(node_queue, StyleActionOrigin::Active);
            style_callback(&mut style);
        }

        if is_active {
            node_visual.clear_normal_timer();
            node_visual.clear_hover_timer();
            node_visual.set_active();
        } else if is_hovered {
            node_visual.clear_normal_timer();
            node_visual.clear_active_timer();
            node_visual.set_hover();
        } else {
            node_visual.clear_hover_timer();
            node_visual.clear_active_timer();
            node_visual.set_normal();
        }

        if let Some(timer) = node_visual.get_current_timer()
            && let Some(start) = node_visual.get_current_timer_start()
            && (Instant::now() < start + timer)
        {
            return;
        } else {
            node_visual.clear_current_timer();
        }

        while let Some(style_action) = node_queue.pop_front::<StyleAction>() {
            match style_action {
                StyleAction::BGColor { origin, color } => {
                    node_visual.set_bg_color(color, origin);
                }
                StyleAction::BGImage { origin, image } => {
                    node_visual.set_bg_image(image, origin);
                }
                StyleAction::Layer { origin, layer } => {
                    node_visual.set_layer(layer, origin);
                }
                StyleAction::BorderRadius { origin, radius } => {
                    node_visual.set_border_radius(radius, origin)
                }
                StyleAction::Scale { origin, x, y } => {
                    node_visual.set_scale(x, y, origin);
                }
                StyleAction::Size {
                    origin,
                    width,
                    height,
                } => {
                    node_visual.set_size(width, height, origin);
                }
                StyleAction::TextContent { origin, content } => {
                    node_visual.set_content(content, origin);
                }
                StyleAction::TextColor { origin, color } => {
                    node_visual.set_text_color(color, origin);
                }
                StyleAction::TextAlign { origin, align } => {
                    node_visual.set_text_align(align, origin);
                }
                StyleAction::Font { origin, font } => {
                    node_visual.set_font(font, origin);
                }
                StyleAction::FontSize { origin, size } => {
                    node_visual.set_font_size(size, origin);
                }
                StyleAction::FontStyle { origin, style } => {
                    node_visual.set_font_style(style, origin);
                }
                StyleAction::Wait { ms } => {
                    node_visual.set_timer(ms);
                    break;
                }
                StyleAction::Position {
                    origin,
                    position,
                    absolute,
                } => {
                    if let Some(value) = position {
                        node_visual.set_position(value, origin);
                    }
                    if let Some(value) = absolute {
                        if value {
                            node_visual.set_position_absolute(origin);
                        } else {
                            node_visual.set_position_relative(origin);
                        }
                    }
                }
            }
        }
        if node_visual.texture_required(NodeTexture::Text) {
            self.texture_handler.create_texture(id, NodeTexture::Text);
        }

        if is_active {
            node_visual.set_active();
        } else if is_hovered {
            node_visual.set_hover();
        } else {
            node_visual.set_normal();
        }
        node_state.is_active = is_active;
        node_state.is_hovered = is_hovered;
    }

    fn process_node_destruction_queue(&mut self, id: NodeID) {
        let node = self.get_node(id).unwrap();
        if node.is_destruction_requested() {
            let family = node
                .get_family(id)
                .expect("Invariant violation: nodes contains an invalid ID");
            for familiar in family {
                if !self.pending_nodes_handler.destroyed_contains(familiar) {
                    self.pending_nodes_handler.push_destroyed(familiar);
                }
            }
        }
    }

    fn process_node_tree_actions(&mut self, id: NodeID) {
        let mut storage = self.storage();
        let node_queue = storage.queue.get_unchecked_mut(id);
        let mut tree = storage.tree;
        while let Some(Action::Tree {
            action: tree_action,
            ..
        }) = node_queue.pop_front_action::<NodeTreeAction>()
        {
            match tree_action {
                NodeTreeAction::AddChild { child } => {
                    tree.get_unchecked_mut(id).add_child(child);
                    tree.get_unchecked_mut(child).set_parent(Some(id));
                }
                NodeTreeAction::RemoveChild { child } => {
                    tree.get_unchecked_mut(id).remove_child(child);
                    tree.get_unchecked_mut(child).set_parent(None);
                }
                NodeTreeAction::SetParent { parent } => {
                    tree.get_unchecked_mut(id).set_parent(parent);
                }
            }
        }
    }
}
impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
