use crate::{
    color::Color,
    style::{Attributes, Style},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellFormat {
    Standard,
    Twoxel,
    Octad,
    Blocktad,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub style: Style,
    pub format: CellFormat,
}

impl Cell {
    pub const EMPTY: Cell = Cell {
        ch: ' ',
        style: Style::EMPTY,
        format: CellFormat::Standard,
    };
}
