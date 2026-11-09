use crate::error::PrismaError;
use crate::resources::manager::ResourceManager;
use sdl3::sys::render::{SDL_DestroyTexture, SDL_Texture};
use sdl3_ttf_sys::ttf::{TTF_CloseFont, TTF_Font};
use std::{
    fmt::Debug,
    hash::Hash,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::resources::macros::define_resource;

define_resource!(
    Font, FontID, FontOptions {
        get_options: {
            size: u16,
        },
        manager => font_manager,
        drop => TTF_CloseFont,
        raw: *mut TTF_Font,
        size: u16,
        name: String,
    }
    PrismaTexture, TextureID, TextureOptions {
        manager => texture_manager,
        drop => SDL_DestroyTexture,
        raw: *mut SDL_Texture,
        width: u32,
        height: u32,
    }
);

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

pub trait Resource: Sized {
    const TYPE: ResourceType;
    type ResourceID: ResourceID<Resource = Self>;
    type GetOptions;

    fn remove(id: Self::ResourceID, manager: &mut ResourceManager) -> Result<Self, PrismaError>;
}

pub trait ResourceID: Sized + Eq + Hash + Copy + Clone {
    type Resource: Resource<ResourceID = Self>;

    fn next() -> Self {
        Self::id(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
    fn id(id: u32) -> Self;
    fn raw(&self) -> u32;
    fn cast<T>(id: T) -> Self
    where
        T: ResourceID;
}

pub trait ResourceGetter<T: Resource> {
    fn get_resource(
        &mut self,
        id: T::ResourceID,
        options: T::GetOptions,
    ) -> Result<&mut T, PrismaError>;

    fn take_resource(
        &mut self,
        id: T::ResourceID,
        options: T::GetOptions,
    ) -> Result<T, PrismaError>;
}
