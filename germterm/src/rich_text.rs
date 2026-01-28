use std::sync::Arc;

use bitflags::bitflags;

use crate::color::Color;

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Attributes: u8 {
        // Standard crossterm & terminal flags
        const BOLD          = 0b_0000_0001;
        const ITALIC        = 0b_0000_0010;
        const UNDERLINED    = 0b_0000_0100;
        const HIDDEN        = 0b_0000_1000;
        // Special renderer flags
        /// Incompatible with OCTAD
        const TWOXEL        = 0b_0001_0000;
        /// Incompatible with TWOXEL
        const OCTAD         = 0b_0010_0000;
        // Erases the cell.

        // This will cause the cell's `ch`, `fg` and `bg` to be ignored completely.

        // The `fg` and `bg` values passed to `crossterm` will be set to `None` and `ch` will be set to a blank space.
        // const ERASE_CELL    = 0b_0100_0000;
    }
}

#[derive(Clone)]
pub struct RichText {
    pub text: Arc<String>,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub attributes: Attributes,
}

impl RichText {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: Arc::new(text.into()),
            fg: Some(Color::WHITE),
            bg: None,
            attributes: Attributes::empty(),
        }
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn attributes(mut self, attributes: Attributes) -> Self {
        self.attributes = attributes;
        self
    }
}

impl From<String> for RichText {
    fn from(s: String) -> Self {
        RichText::new(s).fg(Color::WHITE)
    }
}

impl<'a> From<&'a str> for RichText {
    fn from(s: &'a str) -> Self {
        RichText::new(s).fg(Color::WHITE)
    }
}
