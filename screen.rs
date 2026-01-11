use std::{ops::Deref, str::Chars};

use crate::engine::{color::Color, draw::DrawCall, rich_text::Attributes};

#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct TerminalCell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub attributes: Attributes,
}

pub struct ScreenBuffer(pub Vec<TerminalCell>);

impl Deref for ScreenBuffer {
    type Target = [TerminalCell];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Screen {
    pub cols: u16,
    pub rows: u16,
    pub current_buffer: ScreenBuffer,
    pub old_buffer: ScreenBuffer,
    /// (x, y, terminal_cell)
    pub buffer_diffs: Vec<(u16, u16, TerminalCell)>,
}

impl Screen {
    pub fn new(cols: u16, rows: u16) -> Self {
        let vec_capacity: usize = (cols * rows) as usize;

        Screen {
            cols,
            rows,
            current_buffer: ScreenBuffer(vec![
                TerminalCell {
                    ch: ' ',
                    fg: Color::WHITE,
                    bg: Color::BLACK,
                    attributes: Attributes::empty(),
                };
                vec_capacity
            ]),
            old_buffer: ScreenBuffer(vec![
                TerminalCell {
                    ch: ' ',
                    // Intentional `fg` difference to force the first frame redraw
                    fg: Color::BLACK,
                    bg: Color::BLACK,
                    attributes: Attributes::empty(),
                };
                vec_capacity
            ]),
            buffer_diffs: Vec::with_capacity(vec_capacity),
        }
    }
}

pub fn compose_buffer(buffer: &mut ScreenBuffer, draw_calls: &[DrawCall], cols: u16, rows: u16) {
    let (cols, rows) = (cols as i16, rows as i16);

    for draw_call in draw_calls {
        let mut x: i16 = draw_call.x;
        let y: i16 = draw_call.y;

        let is_oob_left: bool = x < 0;
        let is_oob_top: bool = y < 0;
        let is_oob_right: bool = x >= cols;
        let is_oob_bottom: bool = y >= rows;

        if is_oob_right || is_oob_top || is_oob_bottom {
            continue;
        }

        let mut chars: Chars<'_> = draw_call.rich_text.text.chars();

        if is_oob_left {
            for _ in 0..(-x) {
                chars.next();
            }
            x = 0;
        }

        let row_start_index: usize = (y as usize) * (cols as usize);

        for ch in chars {
            if x >= cols {
                break;
            }

            let cell_index: usize = row_start_index + (x as usize);
            let cell: &mut TerminalCell = &mut buffer.0[cell_index];
            cell.ch = ch;

            x += 1;
        }
    }
}

pub fn diff_buffers(
    buffer_diffs: &mut Vec<(u16, u16, TerminalCell)>,
    current_buffer: &ScreenBuffer,
    old_buffer: &ScreenBuffer,
    cols: u16,
) {
    let cols: usize = cols as usize;

    buffer_diffs.clear();

    let row_pairs = old_buffer.chunks(cols).zip(current_buffer.chunks(cols));

    for (y, (old_row, new_row)) in row_pairs.enumerate() {
        let y: u16 = y as u16;
        let cell_pairs = old_row.iter().zip(new_row.iter());

        for (x, (old_cell, new_cell)) in cell_pairs.enumerate() {
            let x: u16 = x as u16;

            if old_cell != new_cell {
                buffer_diffs.push((x, y, *new_cell));
            }
        }
    }
}
