use crate::{
    node::component::{StyleTextureSeed, StyleTextureType},
    resources::{FontID, ResourceID, TextureID},
    util::{
        Color,
        macros::{style_category, style_texture_category},
    },
};

#[derive(Clone, PartialEq, Debug)]
pub enum FontStyle {
    Bold,
    Italic,
    Normal,
}

#[derive(Clone, PartialEq, Debug)]
pub enum TextAlign {
    Start,
    Center,
    End,
}

pub trait StyleCategory {}
style_category!(
    Background {
        texture: Option<TextureID> => None,
        color: Color => Color::TRANSPARENT,
    }
    Padding {
        top: u32 => 0,
        left: u32 => 0,
        bottom: u32 => 0,
        right: u32 => 0,
    }
    Border {
        radius: u16 => 0,
        top: u32 => 0,
        left: u32 => 0,
        bottom: u32 => 0,
        right: u32 => 0,
        color: Color => Color::TRANSPARENT,
    }
    Margin {
        top: u32 => 0,
        left: u32 => 0,
        bottom: u32 => 0,
        right: u32 => 0,
    }
    Text {
        content: String => String::new(),
        font_family: FontID => FontID::none(),
        font_size: u16 => 24,
        font_style: FontStyle => FontStyle::Normal,
        color: Color => Color::BLACK,
        align: TextAlign => TextAlign::Start,
    }
    Transform {
        x: i32 => 0,
        y: i32 => 0,
        layer: usize => 0,
        rotation: f32 => 0.0,
        scale_x: f32 => 1.0,
        scale_y: f32 => 1.0,
    }
);

style_texture_category!(Background, Text,);
