macro_rules! bubbles_by_default {
    ($value:expr) => {
        $value
    };
    () => {
        false
    };
}
pub(crate) use bubbles_by_default;

macro_rules! define_event {
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
        pub trait EventData: Sized {
            const TYPE: EventType;

            fn cast(event: Event) -> Option<Self>;
        }

        #[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
        pub struct EventCallbackID(u32);
        static NEXT_LISTENER_ID: AtomicU32 = AtomicU32::new(0);
        impl EventCallbackID {
            pub(crate) fn next() -> Self {
                Self(NEXT_LISTENER_ID.fetch_add(1, Ordering::Relaxed))
            }
        }
        impl Display for EventCallbackID {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        pub(crate) struct EventCallback {
            pub callback: Box<dyn FnMut(&mut EventContext) + 'static>,
        }
        impl EventCallback {
            pub fn new<F>(callback: F) -> Self
            where
                F: FnMut(&mut EventContext) + 'static,
            {
                Self {
                    callback: Box::new(callback),
                }
            }

            pub fn execute(&mut self, context: &mut EventContext) {
                (self.callback)(context);
            }
        }

        #[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
        pub enum EventKind {
            $(
                $family,
            )*
        }

        #[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
        pub enum EventType {
            $(
                $family($event_type_enum),
            )*
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum Event {
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
                }
            }

            pub fn get_kind(&self) -> EventKind {
                match self {
                    $(
                        Self::$family(..) => EventKind::$family,
                    )*
                }
            }
        }

        impl Event {
            pub fn get_type(&self) -> EventType {
                match self {
                    $(
                        Event::$family(event) => EventType::$family(event.event_type()),
                    )*
                }
            }

            pub fn get_kind(&self) -> EventKind {
                match self {
                    $(
                        Event::$family { .. } => EventKind::$family,
                    )*
                }
            }
        }


    };
}
pub(crate) use define_event;

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
