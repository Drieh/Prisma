macro_rules! define_resource {
    (
        $(
            $resource:ident, $resource_id:ident, $resource_options:ident {
                $(
                    get_options: {
                        $(
                            $opt_field:ident: $opt_ty:ty,
                        )+
                    },
                )?
                manager => $manager:ident,
                drop => $drop:path,
                raw: $raw_ty:ty,
                $(
                    $field:ident: $ty:ty,
                )*
            }
        )*
    ) => {
        #[derive(Debug, PartialEq, Hash, Eq, Copy, Clone)]
        pub enum ResourceType {
            $(
                $resource,
            )*
        }
        $(

            pub struct $resource_options {
                $(
                    $(
                        pub $opt_field: $opt_ty,
                    )+
                )?
            }

            pub struct $resource {
                raw: $raw_ty,
                $(
                    pub $field: $ty,
                )*
            }

            impl $resource {
                pub fn new(
                    raw: $raw_ty,
                    $(
                        $field: $ty,
                    )*
                ) -> Self {
                    Self {
                        raw,
                        $(
                            $field,
                        )*
                    }
                }

                pub fn raw(&self) -> $raw_ty {
                    self.raw
                }

            }

            impl Resource for $resource {
                const TYPE: ResourceType = ResourceType::$resource;
                type ResourceID = $resource_id;
                type GetOptions = $resource_options;
            }

            impl Drop for $resource {
                fn drop(&mut self) {
                    println!("DROPPING RESOURCE {:?} type {:?}", self.raw(), $resource::TYPE);
                    if !self.raw().is_null() {
                        unsafe {
                            $drop(self.raw());
                        }
                    }
                }
            }

            #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
            pub struct $resource_id {
                id: u32,
            }

            impl ResourceID for $resource_id {
                type Resource = $resource;

                fn id(id: u32) -> Self {
                    if id == 0 {
                        panic!("The value 0 represents a non-valid ID!")
                    }
                    Self { id }
                }

                fn none() -> Self {
                    Self { id: 0 }
                }

                fn raw(&self) -> u32 {
                    self.id
                }

                fn cast<T>(id: T) -> Self where T: ResourceID {
                    Self {id: id.raw()}
                }
            }
        )*
    };
}

pub(crate) use define_resource;

macro_rules! define_resource_managers {
    (
        $(
            $manager:ident: $resource:ident,
        )+$(,)?
    ) => {
        $(
            pub struct $manager<T: Resource> {
                storage: HashMap<T::ResourceID, T>,
                registry: ResourceRegistry<T>,
            }

            impl $manager<$resource> {
                pub fn new() -> Self {
                    Self {
                        storage: HashMap::new(),
                        registry: ResourceRegistry::new()
                    }
                }
            }
        )+
    };
}

pub(crate) use define_resource_managers;
