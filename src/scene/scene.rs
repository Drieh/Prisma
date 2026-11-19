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
use crate::node::component::StyleTextureType;
use crate::render::Renderer;
use crate::resources::ResourceID;
use crate::resources::TextureID;
use crate::resources::manager::ResourceManager;
use crate::scene::Node;
use crate::scene::NodeID;
use crate::scene::handler::CloseHandler;
use crate::scene::handler::PendingNodesHandler;
use crate::scene::storage::{NodeStorage, StorageHandler};
use crate::util::Background;
use crate::util::Color;
use crate::util::Size;
use crate::util::Text;
use std::time::Instant;

/// Represents a UI scene displayed inside an window.
///
/// A scene owns all nodes and processes events within its window.
pub struct Scene {
    pub color: Color,

    nodes: NodeStorage,

    size: Size,

    event_manager: EventManager,

    pending_nodes_handler: PendingNodesHandler,
    close_handler: CloseHandler,
}
impl Scene {
    pub fn new() -> Self {
        Self {
            color: Color::rgb(255, 255, 255),
            size: Size {
                width: 0,
                height: 0,
            },
            nodes: NodeStorage::new(),
            event_manager: EventManager::new(),
            pending_nodes_handler: PendingNodesHandler::new(),
            close_handler: CloseHandler::new(),
        }
    }

    pub fn storage(&mut self) -> StorageHandler<'_> {
        self.nodes.storage()
    }

    /// Creates a new node and returns a mutable [`NodeView`] to it.
    ///
    /// The returned view can be used to configure the node before or after Prisma is running.
    pub fn new_node(&mut self) -> Node<'_> {
        let new_node = self.nodes.new_node();
        self.pending_nodes_handler.push_created(new_node.get_id());
        new_node
    }

    /// Returns `true` if the scene contains a node with the given ID.
    pub fn contains(&mut self, id: NodeID) -> bool {
        self.nodes.contains(id)
    }

    /// Returns a list of all available nodes.
    pub fn get_nodes(&self) -> Vec<NodeID> {
        self.nodes.get_nodes()
    }

    /// Returns a list of all nodes, including nodes with [DisplayLayout::None].
    pub fn get_all_nodes(&self) -> Vec<NodeID> {
        self.nodes.get_all_nodes()
    }

    /// Returns a [`NodeView`] of the given [`NodeID`].
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeNotFound`] if there is no node for the given [`NodeID`].
    pub fn get_node(&mut self, id: NodeID) -> Result<Node<'_>, PrismaError> {
        self.nodes.get_node(id)
    }

    /// Returns `true` if window is about to close.
    pub fn is_quitting(&self) -> bool {
        self.close_handler.is_quitting()
    }

    fn process_node_textures(
        &mut self,
        id: NodeID,
        resources: &mut ResourceManager,
        renderer: &Renderer,
    ) -> Result<(), PrismaError> {
        let StorageHandler { mut visual, .. } = self.nodes.storage();

        let node_visual = visual.get_unchecked_mut(id);
        let state = node_visual.current_visual_state();
        let req_list = [StyleTextureType::Background, StyleTextureType::Text];

        for req in req_list.clone() {
            let texture_required = match req {
                StyleTextureType::Background => {
                    let background = node_visual.get_background();
                    let cached_seed = node_visual
                        .get_texture_cache(req)
                        .and_then(|cached| Some(cached.seed::<Background>()))
                        .unwrap_or(Background::empty());

                    cached_seed != background
                }
                StyleTextureType::Text => {
                    let text = node_visual.get_text();
                    let cached_seed = node_visual
                        .get_texture_cache(req)
                        .and_then(|cached| Some(cached.seed::<Text>()))
                        .unwrap_or(Text::empty());

                    cached_seed != text
                }
            };

            if texture_required {
                println!("texture_required!");
                if let Some(texture) = node_visual.get_texture_cache(req) {
                    if texture.id() != TextureID::none() {
                        resources.remove_user(texture.id(), id)?;
                    }
                }
                match req {
                    StyleTextureType::Text => {
                        if node_visual.get_text().get_content().is_empty() {
                            node_visual.set_texture_cache(req, TextureID::none(), state);
                            continue;
                        }

                        let texture = renderer.create_text_texture(node_visual, resources)?;
                        let text_size = Size {
                            width: texture.width,
                            height: texture.height,
                        };
                        node_visual.set_inline_size(text_size, state);
                        let texture_id = resources.register_internal(texture)?;
                        resources.add_user(texture_id, id)?;
                        node_visual.set_texture_cache(req, texture_id, state);
                    }
                    StyleTextureType::Background => {
                        // provisional
                        node_visual.set_texture_cache(req, TextureID::none(), state);
                    }
                }
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
        self.size = renderer.get_viewport_size();

        for id in self.get_all_nodes() {
            self.process_node_tree_actions(id);
            self.process_event_listener_actions(id);
            self.process_node_style_actions(id);
            self.process_node_destruction_queue(id);
            self.process_node_textures(id, resources, renderer)?;
        }

        let mut storage = self.nodes.storage();
        for id in self.pending_nodes_handler.take_destroyed() {
            let node_visual = storage.visual.get_unchecked(id);
            for texture in node_visual.get_all_textures() {
                resources.remove_user(texture, id)?;
            }
            self.event_manager.clear_node_listeners(id);
            storage.remove_context(id);
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

        node_visual.viewport = self.size;

        let is_hovered = self.event_manager.is_node_hovered(id);
        let is_active = self.event_manager.is_node_active(id);

        if is_active {
            node_visual.clear_normal_timer();
            node_visual.clear_hover_timer();
        } else if is_hovered {
            node_visual.clear_normal_timer();
            node_visual.clear_active_timer();
        } else {
            node_visual.clear_hover_timer();
            node_visual.clear_active_timer();
        }

        if let Some(timer) = node_visual.get_current_timer()
            && let Some(start) = node_visual.get_current_timer_start()
            && (Instant::now() < start + timer)
        {
            return;
        } else {
            node_visual.clear_current_state_timer();
        }

        while let Some(style_action) = node_queue.pop_front::<StyleAction>() {
            match style_action {
                StyleAction::Border {
                    origin,
                    mut callback,
                } => {
                    let mut value = node_visual.get_border();
                    callback(&mut value);
                    node_visual.set_border(value, origin);
                }
                StyleAction::Background {
                    origin,
                    mut callback,
                } => {
                    let mut value = node_visual.get_background();
                    callback(&mut value);
                    node_visual.set_background(value, origin);
                }
                StyleAction::Transform {
                    origin,
                    mut callback,
                } => {
                    let mut value = node_visual.get_transform();
                    callback(&mut value);
                    node_visual.set_transform(value, origin);
                }
                StyleAction::Text {
                    origin,
                    mut callback,
                } => {
                    let mut value = node_visual.get_text();
                    callback(&mut value);
                    node_visual.set_text(value, origin);
                }
                StyleAction::Padding {
                    origin,
                    mut callback,
                } => {
                    let mut value = node_visual.get_padding();
                    callback(&mut value);
                    node_visual.set_padding(value, origin);
                }
                StyleAction::Margin {
                    origin,
                    mut callback,
                } => {
                    let mut value = node_visual.get_margin();
                    callback(&mut value);
                    node_visual.set_margin(value, origin);
                }

                StyleAction::Wait { ms } => {
                    node_visual.wait(ms);
                    break;
                }
                StyleAction::Position { origin, position } => {
                    node_visual.set_position(position, origin);
                }
                StyleAction::Display { origin, display } => {
                    node_visual.set_display(display, origin);
                }
                StyleAction::Size { origin, size } => {
                    node_visual.set_size(size, origin);
                }
            }

            /*
            self.texture_handler
                .eval_texture(id, StyleTextureType::Background);
             */
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
            }
        }
    }
}
impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
