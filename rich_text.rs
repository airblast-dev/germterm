use bitflags::bitflags;

use crate::engine::color::Color;

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Attributes: u8 {
        const BOLD       = 0b000_0001;
        const ITALIC     = 0b000_0010;
        const UNDERLINED = 0b000_0100;
        const HIDDEN     = 0b000_1000;
    }
}

#[derive(Clone)]
pub struct RichText {
    pub text: String,
    pub fg: Color,
    pub bg: Color,
    pub attributes: Attributes,
}

impl RichText {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            fg: Color::WHITE,
            bg: Color::CLEAR,
            attributes: Attributes::empty(),
        }
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = color;
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = color;
        self
    }

    pub fn attributes(mut self, attributes: Attributes) -> Self {
        self.attributes = attributes;
        self
    }
}

impl From<String> for RichText {
    fn from(s: String) -> Self {
        RichText::new(s)
    }
}

impl<'a> From<&'a str> for RichText {
    fn from(s: &'a str) -> Self {
        RichText::new(s)
    }
}
