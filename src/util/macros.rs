macro_rules! style_category {
    (
        $(
            $category:ident {
                $(
                    $field:ident: $ty:ty => $default:expr,
                )*
            }
        )*
    ) => {
        $(
            #[derive(Clone, PartialEq, Debug)]
            pub struct $category {
                $(
                    $field: Option<$ty>,
                )*
            }

            impl StyleCategory for $category {

            }

            impl $category {
                pub fn default() -> Self {
                    Self {
                        $(
                            $field: Some($default),
                        )*
                    }
                }

                pub fn empty() -> Self {
                    Self {
                        $(
                            $field: None,
                        )*
                    }
                }

                pub fn merge(&mut self, layer: &Self) {
                    $(
                        if let Some(value) = &layer.$field {
                            self.$field = Some(value.clone());
                        }
                    )*
                }

                $(
                    pub fn $field(&mut self, value: $ty) -> &mut Self {
                        self.$field = Some(value.into());
                        self
                    }
                )*
            }
            prisma_macros::generate_style_getters!(
                $category {
                    $(
                        $field: $ty => $default,
                    )*
                }
            );
        )*
    };
}
pub(crate) use style_category;

macro_rules! style_texture_category {
    (
        $(
            $category:ident,
        )*$(,)?
    ) => {
        pub trait StyleTextureCategory {
            fn from_seed(seed: &StyleTextureSeed) -> Self
            where
                Self: Sized;
        }

        $(
            impl StyleTextureCategory for $category {
            fn from_seed(seed: &StyleTextureSeed) -> Self
            where
                Self: Sized,
            {
                match seed {
                    StyleTextureSeed::$category(value) => value.clone(),
                    _ => panic!("Invariant violated.")
                }
            }
        }
        )*
    };
}
pub(crate) use style_texture_category;
