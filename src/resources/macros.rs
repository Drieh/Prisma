macro_rules! define_resources {
    (
        $(
            $resource:ident, $resource_id:ident, $resource_count:ident {
                $(
                    $field:ident: $ty:ty;
                )*
            }
        )*
    ) => {
        pub enum ResourceType {
            $(
                $resource,
            )*
        }
        $(
            pub struct $resource {

                $(
                    $field: $ty,
                )*
            }

            impl Resource for $resource {
                const TYPE: ResourceType = ResourceType::$resource;
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
            pub struct $resource_id {
                id: u32,
            }

            static $resource_count: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

            impl $resource_id {
                pub fn id(id: u32) -> Self {
                    Self { id }
                }
            }
            impl std::fmt::Display for $resource_id {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.id)
                }
            }
        )*
    };
}

pub(crate) use define_resources;
