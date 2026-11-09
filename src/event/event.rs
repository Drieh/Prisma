use crate::{
    event::{EventContext, macros::*},
    node::NodeID,
    util::Position,
};
use std::fmt::Display;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
    Unknown,
}

define_event! {
    System as SystemEvent, SystemEventType {
        Quit as Quit {}
        Close as CloseRequest {}
        CancelClose as CancelClose {}
    }
    Window as WindowEvent, WindowEventType
    {
        WindowCloseRequest as WindowCloseRequest {
            window_id: u32,
        }
        WindowMove as WindowMove {
            window_id: u32,
            position: Position,
        }
        WindowResized as WindowResized {
            window_id: u32,
            width: i32,
            height: i32,
        }
    }
    Lifecycle as LifecycleEvent, LifecycleEventType
    {

        LifecycleCreation as NodeCreation {
            target: NodeID,
        }
        LifecycleUpdate as NodeUpdate {
            target: NodeID,
        }
        LifecycleDestruction as NodeDestruction {
            target: NodeID,
        }
    }
    Mouse as MouseEvent, MouseEventType
    {
        MouseClick as Click {
            position: Position,
            mouse_btn: MouseButton,
        }
        MouseUp as MouseUp {
            position: Position,
            mouse_btn: MouseButton,
        }
        MouseDown as MouseDown {
            position: Position,
            mouse_btn: MouseButton,
        }
        MouseMove as MouseMove {
            position: Position,
        }
        MouseDragStart as DragStart {
            position: Position,
            mouse_btn: MouseButton,
        }
        MouseDrag as Drag {
            position: Position,
            mouse_btn: MouseButton,
        }
        MouseDragEnd as DragEnd {
            position: Position,
            mouse_btn: MouseButton,
        }
        MouseEnter as MouseEnter {
            position: Position,
        }
        MouseLeave as MouseLeave {
            position: Position,
        }
    }
}
