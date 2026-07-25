use std::fmt;

use crate::scene::nodes::NodeID;

#[derive(Debug)]
pub enum PrismaError {
    InitError(String),
    InvalidTree((NodeID, NodeID)),
    NodeStateNotFound((NodeID, String)),
    NodeComponentNotFound(NodeID),
    NodeNotFound(NodeID),
    RenderError(String),
}
impl fmt::Display for PrismaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrismaError::InitError(msg) => {
                write!(f, "Prisma error: {msg}")
            }
            PrismaError::RenderError(msg) => {
                write!(f, "Render error: {msg}")
            }
            PrismaError::NodeNotFound(id) => {
                write!(f, "Node {id} not found")
            }
            PrismaError::NodeStateNotFound((id, key)) => {
                write!(f, "State {key} not found for node {id}")
            }
            PrismaError::NodeComponentNotFound(id) => {
                write!(f, "Component not found for node: {id}")
            }
            PrismaError::InvalidTree((parent, child)) => {
                write!(f, "Invalid tree for parent {parent} and child {child}")
            }
        }
    }
}

impl std::error::Error for PrismaError {}
