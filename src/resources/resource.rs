use sdl3_ttf_sys::ttf::TTF_Font;

use crate::resources::macros::define_resources;

define_resources!(
    Font, FontID, NEXT_FONT_ID {
        name: String;
        font: TTF_Font;
    }
    Image, ImageID, NEXT_IMAGE_ID {
        path: String;
    }
);

pub trait Resource {
    const TYPE: ResourceType;
}
