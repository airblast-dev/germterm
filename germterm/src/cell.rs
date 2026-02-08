use crate::{color::Color, metadata::Metadata, rich_text::Attributes};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub attributes: Attributes,
    pub(crate) metadata: Metadata,
}

impl Cell {
    pub const EMPTY: Cell = Cell {
        ch: ' ',
        fg: Color::BLACK,
        bg: Color::BLACK,
        attributes: Attributes::empty(),
        metadata: Metadata::empty(),
    };
}
