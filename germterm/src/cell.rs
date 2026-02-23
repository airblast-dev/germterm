use crate::{color::Color, style::Attributes};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellFormat {
    Standard,
    Twoxel,
    Octad,
    Blocktad,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub attributes: Attributes,
    pub format: CellFormat,
}

impl Cell {
    pub const EMPTY: Cell = Cell {
        ch: ' ',
        fg: Color::CLEAR,
        bg: Color::CLEAR,
        attributes: Attributes::from_bits_truncate(
            Attributes::NO_FG_COLOR.bits() | Attributes::NO_BG_COLOR.bits(),
        ),
        format: CellFormat::Standard,
    };

    pub fn merge(&mut self, other: Self) {
        self.ch = other.ch;
        // TODO: use [`Style`] and call its merge instead beyond this point
        self.attributes |= other.attributes;
        if !other.attributes.contains(Attributes::NO_FG_COLOR) {
            self.attributes &= !Attributes::NO_FG_COLOR;
            self.fg = other.fg;
        }
        if !other.attributes.contains(Attributes::NO_BG_COLOR) {
            self.attributes &= !Attributes::NO_BG_COLOR;
            self.bg = other.bg;
        }
    }
}
