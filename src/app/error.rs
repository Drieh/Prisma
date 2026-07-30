use std::fmt;

use crate::{event::EventKind, scene::NodeID};

#[derive(Debug, PartialEq)]
pub enum PrismaError {
    /// Returned when an error occurs during application initialization.
    ///
    /// Contains the underlying error message.
    InitError(String),

    /// Returned when an invalid parent-child relationship is detected.
    ///
    /// Contains the parent and child node IDs.
    InvalidTree(NodeID, NodeID),

    /// Returned when the node does not contain a state associated with the given key.
    ///
    /// Contains the node ID and the missing state key.
    NodeStateNotFound(NodeID, String),

    /// Returned when a node component is requested using an invalid [`NodeID`].
    ///
    /// Contains the invalid node ID.
    NodeComponentNotFound(NodeID),

    /// Returned when a node with the given ID could not be found.
    ///
    /// Contains the missing node ID.
    NodeNotFound(NodeID),

    /// Returned when an error occurs during rendering.
    ///
    /// Contains the underlying error message.
    RenderError(String),

    UnexpectedEventType(EventKind, EventKind),
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
            PrismaError::NodeStateNotFound(id, key) => {
                write!(f, "State {key} not found for node {id}")
            }
            PrismaError::NodeComponentNotFound(id) => {
                write!(f, "Component not found for node: {id}")
            }
            PrismaError::InvalidTree(parent, child) => {
                write!(f, "Invalid tree for parent {parent} and child {child}")
            }
            PrismaError::UnexpectedEventType(expected, found) => {
                write!(
                    f,
                    "Event types do not matches: expected {expected:?} but found {found:?}"
                )
            }
        }
    }
}

impl std::error::Error for PrismaError {}
