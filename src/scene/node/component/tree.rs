use crate::scene::NodeID;

#[derive(Debug, Clone)]
pub struct NodeTree {
    parent: Option<NodeID>,
    children: Vec<NodeID>,
}
impl NodeTree {
    pub(crate) fn new() -> Self {
        Self {
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn get_parent(&self) -> Option<NodeID> {
        self.parent
    }

    pub fn get_children(&self) -> Vec<NodeID> {
        self.children.clone()
    }

    pub(crate) fn set_parent(&mut self, parent: Option<NodeID>) {
        self.parent = parent;
    }

    pub(crate) fn add_child(&mut self, child_id: NodeID) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    pub(crate) fn remove_child(&mut self, child_id: NodeID) {
        if let Some(i) = self.children.iter().position(|id| *id == child_id) {
            self.children.remove(i);
        }
    }
}
