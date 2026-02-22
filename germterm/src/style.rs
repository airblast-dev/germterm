use crate::color::Color;
use bitflags::bitflags;

bitflags! {
    /// Attributes that can be applied to drawn text.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Attributes: u8 {
        const BOLD          = 0b_00000001;
        const ITALIC        = 0b_00000010;
        const UNDERLINED    = 0b_00000100;
        const HIDDEN        = 0b_00001000;
        const NO_FG_COLOR   = 0b_00010000;
        const NO_BG_COLOR   = 0b_00100000;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(Rust, packed)]
pub struct Style {
    fg: Color,
    bg: Color,
    pub attributes: Attributes,
}

impl Style {
    pub const EMPTY: Self = Style {
        fg: Color(0),
        bg: Color(0),
        attributes: Attributes::from_bits_truncate(
            Attributes::NO_FG_COLOR.bits() | Attributes::NO_BG_COLOR.bits(),
        ),
    };

    pub fn new(fg: Option<Color>, bg: Option<Color>, attr: Attributes) -> Self {
        let mut s = Self::EMPTY;
        s.attributes = attr;
        *s.set_fg(fg).set_bg(bg)
    }

    pub fn set_fg(&mut self, color: Option<Color>) -> &mut Self {
        if let Some(c) = color {
            self.fg = c;
            self.attributes &= !Attributes::NO_FG_COLOR;
        } else {
            self.attributes |= Attributes::NO_FG_COLOR;
        }

        self
    }

    pub fn fg(&self) -> Option<Color> {
        if self.has_fg() {
            Some(self.fg)
        } else {
            None
        }
    }

    #[inline]
    pub fn has_fg(&self) -> bool {
        !self.attributes.contains(Attributes::NO_FG_COLOR)
    }

    pub fn set_bg(&mut self, color: Option<Color>) -> &mut Self {
        if let Some(c) = color {
            self.bg = c;
            self.attributes &= !Attributes::NO_BG_COLOR;
        } else {
            self.attributes |= Attributes::NO_BG_COLOR;
        }

        self
    }

    pub fn bg(&self) -> Option<Color> {
        if self.has_bg() {
            Some(self.bg)
        } else {
            None
        }
    }

    #[inline]
    pub fn has_bg(&self) -> bool {
        !self.attributes.contains(Attributes::NO_BG_COLOR)
    }
}
