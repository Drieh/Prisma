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
    /// Contains the missing state key.
    NodeStateNotFound(String),

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

    /// Returned when an event doen't match the expected event type.
    ///
    /// Contains the expected and found event types.
    UnexpectedEventType(EventKind, EventKind),

    /// Returned when a resource is not found.
    ResourceNotFound(String),
}
impl fmt::Display for PrismaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrismaError::InitError(msg) => {
                write!(f, "Initialization error: {msg}")
            }
            PrismaError::RenderError(msg) => {
                write!(f, "Render error: {msg}")
            }
            PrismaError::NodeNotFound(id) => {
                write!(f, "Node {id} not found")
            }
            PrismaError::NodeStateNotFound(t) => {
                write!(f, "State {t} not found.")
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
            PrismaError::ResourceNotFound(msg) => {
                write!(f, "Resource not found: {msg}")
            }
        }
    }
}

impl std::error::Error for PrismaError {}
