use crate::{
    event::{EventCallback, EventCallbackID, EventType},
    node::{
        NodeID,
        macros::{define_action_family, define_node_actions},
    },
    resources::{
        FontID, TextureID,
        font::{FontStyle, TextAlign},
    },
    util::{Color, Position},
};

define_node_actions!(
    Style as StyleAction {
        BGColor {
            origin: StyleActionOrigin,
            color: Color => Color::rgba(255, 255, 255, 255),
        }
        Layer {
            origin: StyleActionOrigin,
            layer: usize => 0,
        }
        BorderRadius {
            origin: StyleActionOrigin,
            radius: u32 => 0,
        }
        Scale {
            origin: StyleActionOrigin,
            x: f32 => 0.0,
            y: f32 => 0.0,
        }
        Position {
            origin: StyleActionOrigin,
            position: Option<Position> => None,
            absolute: Option<bool> => None,
        }
        Size {
            origin: StyleActionOrigin,
            width: u32 => 0,
            height: u32 => 0,
        }
        TextContent {
            origin: StyleActionOrigin,
            content: String,
        }
        TextColor {
            origin: StyleActionOrigin,
            color: Color,
        }
        TextAlign {
            origin: StyleActionOrigin,
            align: TextAlign,
        }
        Font {
            origin: StyleActionOrigin,
            font: FontID,
        }
        FontSize {
            origin: StyleActionOrigin,
            size: u16,
        }
        FontStyle {
            origin: StyleActionOrigin,
            style: FontStyle,
        }
        BGImage {
            origin: StyleActionOrigin,
            image: TextureID,
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
    Tree as NodeTreeAction {
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
    Text as TextAction {

    },
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleActionOrigin {
    Code,
    Hover,
    Active,
}
