use crate::node::{NodeID, component::NodeTexture};

#[derive(Clone, Copy)]
pub struct TextureRequirement {
    pub node: NodeID,
    pub req: NodeTexture,
}

pub struct TextureHandler {
    queue: Vec<TextureRequirement>,
}
impl TextureHandler {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn create_texture(&mut self, id: NodeID, req: NodeTexture) {
        self.queue.push(TextureRequirement { node: id, req });
    }

    pub fn remove_node(&mut self, id: NodeID) {
        self.queue = self
            .queue
            .iter()
            .filter_map(|req| if req.node == id { None } else { Some(*req) })
            .collect();
    }

    pub fn take_queue(&mut self) -> Vec<TextureRequirement> {
        std::mem::take(&mut self.queue)
    }
}
