use crate::{
    event::{EventCallback, EventCallbackID, EventType},
    node::{
        NodeID,
        component::StyleVisualState,
        macros::{define_action_family, define_node_actions},
    },
    util::{
        Background, Border, DisplayLayout, DisplayPosition, Margin, Padding, Size, Text, Transform,
    },
};

type BackgroundCallback = dyn FnMut(&mut Background) + 'static;
type TransformCallback = dyn FnMut(&mut Transform) + 'static;
type TextCallback = dyn FnMut(&mut Text) + 'static;
type PaddingCallback = dyn FnMut(&mut Padding) + 'static;
type BorderCallback = dyn FnMut(&mut Border) + 'static;
type MarginCallback = dyn FnMut(&mut Margin) + 'static;

define_node_actions!(
    Style as StyleAction {
        Background {
            origin: StyleVisualState,
            callback: Box<BackgroundCallback>,
        }
        Transform {
            origin: StyleVisualState,
            callback: Box<TransformCallback>,
        }
        Text {
            origin: StyleVisualState,
            callback: Box<TextCallback>,
        }
        Padding {
            origin: StyleVisualState,
            callback: Box<PaddingCallback>,
        }
        Border {
            origin: StyleVisualState,
            callback: Box<BorderCallback>,
        }
        Margin {
            origin: StyleVisualState,
            callback: Box<MarginCallback>,
        }
        Wait {
            ms: u64 => 0,
        }
        Position {
            origin: StyleVisualState,
            position: DisplayPosition,
        }
        Display {
            origin: StyleVisualState,
            display: DisplayLayout,
        }
        Size {
            origin: StyleVisualState,
            size: Size,
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
    },
);
