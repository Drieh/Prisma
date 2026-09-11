use crate::{
    event::{EventCallback, EventCallbackID, EventType},
    node::{
        NodeID,
        macros::{define_action_family, define_node_actions},
    },
    util::{Color, Position},
};

define_node_actions!(
    Style as StyleAction {
        BGColor {
            color: Color => Color::rgba(255, 255, 255, 255),
        }
        Layer {
            layer: usize => 0,
        }
        BorderRadius {
            radius: u32 => 0,
        }
        Scale {
            x: f32 => 0.0,
            y: f32 => 0.0,
        }
        Position {
            position: Option<Position> => None,
            absolute: Option<bool> => None,
        }
        Size {
            width: u32 => 0,
            height: u32 => 0,
        }
        Wait {
            ms: u64 => 0,
        }
    },
    EventListener as EventListenerAction {
        Add {
            event_type: EventType,
            callback: EventCallback,
        }
        Remove {
            target: EventCallbackID,
        }
    },
    NodeTree as NodeTreeAction {
        AddChild {
            child: NodeID,
        }
        RemoveChild {
            child: NodeID,
        }
        SetParent {
            parent: Option<NodeID>,
        }
    },
);
