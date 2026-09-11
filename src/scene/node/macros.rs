macro_rules! define_node_actions {
    (
        $(
            $family:ident as $family_enum:ident {
                $(
                    common_field: {
                        $(
                            $common_field:ident: $common_ty:ty $(=> $common_value:expr)?,
                        )+
                    }
                )?
                $(
                    $action:ident {
                        $(
                            $field:ident: $ty:ty $(=> $value:expr)?,
                        )*
                    }
                )*
            },
        )*
    ) => {

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum ActionType {
            $(
                $family,
            )*
        }

        pub trait ActionFamily {
            const TYPE: ActionType;

            fn cast(action: Action) -> Option<Self> where Self: Sized;
            fn into_action(self, origin: ActionOrigin) -> Action where Self: Sized;

        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum ActionOrigin {
            Code,
            Event,
            Visual,
        }

        pub enum Action {
            $(
                $family {
                    action: $family_enum,
                    origin: ActionOrigin,
                },
            )*
        }

        $(
            define_action_family!(
                $family as $family_enum {
                    common: {
                        $(
                            $(
                                $common_field: $common_ty $(=> $common_value)?,
                            )+
                        )?
                    }
                    actions {
                        $(
                            $action {
                                $(
                                    $field: $ty $(=> $value)?,
                                )*
                            }
                        )*
                    }
                }
            );
        )*
    };
}
pub(crate) use define_node_actions;

macro_rules! define_action_family {
    (
        $family:ident as $family_enum:ident {
            common: {
                $(
                    $(
                        $common_field:ident: $common_ty:ty $(=> $common_value:expr)?,
                    )+
                )?
            }
            actions {
                $(
                    $action:ident {
                        $(
                            $field:ident: $ty:ty $(=> $value:expr)?,
                        )*
                    }
                )*
            }
        }
    ) => {
        pub enum $family_enum {
            $(
                $action {
                    $(
                        $field: $ty,
                    )*
                },
            )*
        }

        impl ActionFamily for $family_enum {
            const TYPE: ActionType = ActionType::$family;

            fn cast(action: Action) -> Option<Self> {
                match action {
                    Action::$family { action: family_action, ..} => Some(family_action),
                    _ => None,
                }
            }

            fn into_action(self, origin: ActionOrigin) -> Action {
                Action::$family {
                    action: self,
                    origin
                }
            }
        }

        impl std::fmt::Debug for $family_enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Action from family {}", stringify!($family_enum))
            }
        }
    };
}
pub(crate) use define_action_family;
