use std::fmt;

use crate::scene::NodeID;

#[derive(Debug)]
pub enum PrismaError {
    InitError(String),
    NodeStateNotFound(String),
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
            PrismaError::NodeStateNotFound(key) => {
                write!(f, "Node state {key} not found")
            }
        }
    }
}

impl std::error::Error for PrismaError {}
