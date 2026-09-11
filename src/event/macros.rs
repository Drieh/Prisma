macro_rules! bubbles_by_default {
    ($value:expr) => {
        $value
    };
    () => {
        false
    };
}
pub(crate) use bubbles_by_default;

macro_rules! define_events {
    (
        $(
            $family:ident as $event_enum:ident, $event_type_enum:ident
            {
                $(
                    $variant:ident as $name:ident {
                        $(bubbles_by_default: $bubbles_by_default:expr,)?
                        $(
                            $field:ident : $ty:ty
                        ),* $(,)?
                    }
                )* $(,)?
            }
        )* $(,)?
    ) => {
        #[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
        pub enum EventKind {
            Quit,
            AppCloseRequest,
            CancelAppCloseRequest,
            $(
                $family,
            )*
        }

        #[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
        pub enum EventType {
            Quit,
            AppCloseRequest,
            CancelAppCloseRequest,
            $(
                $family($event_type_enum),
            )*
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum Event {
            Quit,
            AppCloseRequest,
            CancelAppCloseRequest,
            $(
                $family($event_enum),
            )*
        }


                $(
                define_event_family!(
                $family as $event_enum, $event_type_enum
                    {
                        $(
                            $variant as $name {
                                $(
                                    $field: $ty,
                                )*
                            }
                        )*
                    }
                );
            )*
        impl EventType {
            pub fn bubbles_by_default(&self) -> bool {
                match self {
                    $(
                        $(
                            Self::$family($event_type_enum::$variant) => {
                                bubbles_by_default!($($bubbles_by_default)?)
                            },
                        )*
                    )*
                    Self::Quit
                    | Self::AppCloseRequest
                    | Self::CancelAppCloseRequest => false,
                }
            }

            pub fn get_kind(&self) -> EventKind {
                match self {
                    $(
                        Self::$family(..) => EventKind::$family,
                    )*
                    Self::Quit => EventKind::Quit,
                    Self::AppCloseRequest => EventKind::AppCloseRequest,
                    Self::CancelAppCloseRequest => EventKind::CancelAppCloseRequest,
                }
            }
        }

        impl Event {
            pub fn get_type(&self) -> EventType {
                match self {
                    Event::Mouse(event) => EventType::Mouse(event.event_type()),
                    Event::Window(event) => EventType::Window(event.event_type()),
                    Event::Lifecycle(event) => EventType::Lifecycle(event.event_type()),
                    Event::AppCloseRequest => EventType::AppCloseRequest,
                    Event::CancelAppCloseRequest => EventType::CancelAppCloseRequest,
                    Event::Quit => EventType::Quit,
                }
            }

            pub fn get_kind(&self) -> EventKind {
                match self {
                    Event::Mouse { .. } => EventKind::Mouse,
                    Event::Window { .. } => EventKind::Window,
                    Event::Lifecycle { .. } => EventKind::Lifecycle,
                    Event::AppCloseRequest => EventKind::AppCloseRequest,
                    Event::CancelAppCloseRequest => EventKind::CancelAppCloseRequest,
                    Event::Quit => EventKind::Quit,
                }
            }
        }


    };
}
pub(crate) use define_events;

macro_rules! define_event_family {
    (
        $family:ident as $event_enum:ident, $event_type_enum:ident
        {
            $(
                $variant:ident as $name:ident {
                    $($field:ident : $ty:ty),* $(,)?
                }
            )* $(,)?
        }
    ) => {
        #[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
        pub enum $event_type_enum {
            $($variant,)*
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum $event_enum {
            $(
                $variant($name),
            )*
        }

        impl $event_enum {
            pub fn event_type(&self) -> $event_type_enum {
                match self {
                    $(
                        Self::$variant(_) => $event_type_enum::$variant,
                    )*
                }
            }
        }

        $(
            struct_event!(
                $family as $event_enum, $event_type_enum
                $variant as $name
                {
                    $($field: $ty),*
                }
            );
        )*
    };
}
pub(crate) use define_event_family;

macro_rules! struct_event {
    (
        $family:ident as $event_enum:ident, $event_type_enum:ident
        $variant:ident as $name:ident
        {
            $(
                $field:ident : $ty:ty
            ),* $(,)?
        }
    ) => {

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $name {
            $(pub $field: $ty),*
        }
        impl $crate::event::EventData for $name {
            const TYPE: EventType =
                EventType::$family($event_type_enum::$variant);

            fn cast(event: Event) -> Option<Self> {
                match event {
                    Event::$family($event_enum::$variant(data)) => Some(data),
                    _ => None,
                }
            }
        }
    };
}
pub(crate) use struct_event;
