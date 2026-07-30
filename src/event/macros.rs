macro_rules! define_events {
    (
        $(
            $family:ident as $event_enum:ident, $event_type_enum:ident
            {
                $(
                    $variant:ident as $name:ident {
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
