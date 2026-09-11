use std::ops::{Add, AddAssign};

use sdl3::pixels::Color as SdlColor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Color {
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const YELLOW: Self = Self::rgb(225, 255, 0);
    pub const ORANGE: Self = Self::rgb(255, 150, 0);
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub(crate) const fn into_sdl_color(self) -> SdlColor {
        SdlColor::RGBA(self.r, self.g, self.b, self.a)
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.r = (self.r + rhs.r) / 2;
        self.g = (self.g + rhs.g) / 2;
        self.b = (self.b + rhs.b) / 2;
        self.a = (self.a + rhs.a) / 2;
    }
}
