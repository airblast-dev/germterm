use std::{ops::Deref, str::Chars};

use crossterm::style::{
    Attribute as CrosstermAttribute, Attributes as CrosstermAttributes,
    ContentStyle as CrosstermContentStyle,
};

use crate::engine::{
    color::{Color, blend_over},
    draw::DrawCall,
    rich_text::Attributes,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct TerminalCell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub attributes: Attributes,
}

pub struct DiffProduct {
    pub x: u16,
    pub y: u16,
    pub cell: TerminalCell,
}

#[derive(Clone)]
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
    pub diff_products: Vec<DiffProduct>,
}

impl Screen {
    pub fn new(cols: u16, rows: u16) -> Self {
        let vec_capacity: usize = (cols * rows) as usize;
        let empty_buffer: ScreenBuffer = ScreenBuffer(vec![
            TerminalCell {
                ch: ' ',
                fg: Color::WHITE,
                bg: Color::BLACK,
                attributes: Attributes::empty(),
            };
            vec_capacity
        ]);

        Screen {
            cols,
            rows,
            current_buffer: empty_buffer.clone(),
            old_buffer: empty_buffer.clone(),
            diff_products: Vec::with_capacity(vec_capacity),
        }
    }
}

pub fn compose_buffer(buffer: &mut ScreenBuffer, draw_queue: &[DrawCall], cols: u16, rows: u16) {
    #[inline]
    fn compose_ch(old_ch: char, ch: char, fg: Color, bg: Color) -> char {
        if bg.a() == 255 {
            // Opaque bg => Override char
            return ch;
        }

        if old_ch != ' ' && fg.a() < 255 {
            return old_ch;
        }

        ch
    }

    #[inline]
    fn compose_fg(
        old_fg: Color,
        fg: Color,
        old_bg: Color,
        bg: Color,
        old_ch: char,
        ch: char,
    ) -> Color {
        let old_ch_is_invisible: bool = old_ch == ' ' || old_fg.a() == 0;
        let new_ch_is_invisible: bool = ch == ' ' || fg.a() == 0;

        if new_ch_is_invisible {
            // can't blend old_fg with fg => blend old_fg with bg instead
            return blend_over(old_fg, bg);
        }

        if fg.a() == 255 {
            // fg is opaque => no need to blend, can skip
            return fg;
        }

        if old_ch_is_invisible {
            // can't blend old_fg with fg => blend fg with old_bg instead
            return blend_over(old_bg, fg);
        }

        blend_over(old_fg, fg)
    }

    #[inline]
    fn compose_bg(old_bg: Color, bg: Color) -> Color {
        if bg.a() == 0 {
            return old_bg;
        }

        let blend_bg_to_bg: bool = !matches!(bg.a(), 0 | 255);
        if blend_bg_to_bg {
            return blend_over(old_bg, bg);
        }

        bg
    }

    #[inline]
    fn compose_attributes(
        old_attributes: Attributes,
        attributes: Attributes,
        old_ch: char,
        fg: Color,
        bg: Color,
    ) -> Attributes {
        if bg.a() == 255 {
            // Opaque bg => Override attributes
            return attributes;
        }

        if old_ch != ' ' && fg.a() < 255 {
            return old_attributes;
        }

        attributes
    }

    let (cols, rows) = (cols as i16, rows as i16);

    for draw_call in draw_queue {
        let mut x: i16 = draw_call.x;
        let y: i16 = draw_call.y;

        // --- Skipping out of bounds draw calls ---
        let is_oob_left: bool = x < 0;
        let is_oob_top: bool = y < 0;
        let is_oob_right: bool = x >= cols;
        let is_oob_bottom: bool = y >= rows;

        if is_oob_right || is_oob_top || is_oob_bottom {
            continue;
        }

        let mut chars: Chars<'_> = draw_call.rich_text.text.chars();

        // --- Cropping the out of bounds left side chars ---
        if is_oob_left {
            for _ in 0..(-x) {
                chars.next();
            }
            x = 0;
        }

        let row_start_index: usize = (y as usize) * (cols as usize);
        let remaining_cols: usize = (cols - x).max(0) as usize;

        for (x_offset, ch) in chars.take(remaining_cols).enumerate() {
            let cell_index: usize = row_start_index + x as usize + x_offset;
            let buffer_cell: &mut TerminalCell = &mut buffer.0[cell_index];

            let fg: Color = draw_call.rich_text.fg;
            let bg: Color = draw_call.rich_text.bg;
            let attributes: Attributes = draw_call.rich_text.attributes;

            let new_buffer_cell: TerminalCell = TerminalCell {
                ch: compose_ch(buffer_cell.ch, ch, fg, bg),
                fg: compose_fg(buffer_cell.fg, fg, buffer_cell.bg, bg, buffer_cell.ch, ch),
                bg: compose_bg(buffer_cell.bg, bg),
                attributes: compose_attributes(
                    buffer_cell.attributes,
                    attributes,
                    buffer_cell.ch,
                    fg,
                    bg,
                ),
            };

            buffer.0[cell_index] = new_buffer_cell
        }
    }
}

pub fn diff_buffers(
    diff_products: &mut Vec<DiffProduct>,
    current_buffer: &ScreenBuffer,
    old_buffer: &ScreenBuffer,
    cols: u16,
) {
    let cols: usize = cols as usize;

    diff_products.clear();

    let row_pairs = old_buffer.chunks(cols).zip(current_buffer.chunks(cols));

    for (y, (old_row, new_row)) in row_pairs.enumerate() {
        let y: u16 = y as u16;
        let cell_pairs = old_row.iter().zip(new_row.iter());

        for (x, (old_cell, new_cell)) in cell_pairs.enumerate() {
            let x: u16 = x as u16;

            if old_cell != new_cell {
                diff_products.push(DiffProduct {
                    x,
                    y,
                    cell: *new_cell,
                });
            }
        }
    }
}

pub fn build_crossterm_content_style(cell: &TerminalCell) -> crossterm::style::ContentStyle {
    let fg_color: crossterm::style::Color = crossterm::style::Color::Rgb {
        r: cell.fg.r(),
        g: cell.fg.g(),
        b: cell.fg.b(),
    };

    let bg_color: crossterm::style::Color = crossterm::style::Color::Rgb {
        r: cell.bg.r(),
        g: cell.bg.g(),
        b: cell.bg.b(),
    };

    let attributes = [
        (Attributes::BOLD, CrosstermAttribute::Bold),
        (Attributes::ITALIC, CrosstermAttribute::Italic),
        (Attributes::UNDERLINED, CrosstermAttribute::Underlined),
        (Attributes::HIDDEN, CrosstermAttribute::Hidden),
    ]
    .iter()
    .fold(
        CrosstermAttributes::none(),
        |crossterm_attrs, (attribute, crossterm_attribute)| {
            if cell.attributes.contains(*attribute) {
                crossterm_attrs | *crossterm_attribute
            } else {
                crossterm_attrs
            }
        },
    );

    CrosstermContentStyle {
        foreground_color: Some(fg_color),
        background_color: Some(bg_color),
        underline_color: None,
        attributes,
    }
}
