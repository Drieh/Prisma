use crate::{resources::resource::FontID, util::Color};

pub struct TextAlign {}

pub struct NodeText {
    pub content: String,
    pub font: FontID,
    pub size: u16,
    pub color: Color,
    pub align: TextAlign,
}
impl NodeText {
    pub fn new() -> Self {
        Self {
            content: "".to_string(),
            font: FontID::id(0),
            size: 12,
            color: Color::rgb(0, 0, 0),
            align: TextAlign {},
        }
    }
}
