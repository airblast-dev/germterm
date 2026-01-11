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
                    // Intentional `fg` and `bg` difference to force the first frame redraw
                    // TODO: THIS DOES NOT WORK, STILL NEED TO MANUALLY ADD THE DRAWCALLS FOR FILLING THE SCREEN
                    // ELSE SHIT DOES NOT BLEND!!!
                    fg: Color::BLACK,
                    bg: Color::WHITE,
                    attributes: Attributes::empty(),
                };
                vec_capacity
            ]),
            buffer_diffs: Vec::with_capacity(vec_capacity),
        }
    }
}

pub fn compose_buffer(buffer: &mut ScreenBuffer, draw_calls: &[DrawCall], cols: u16, rows: u16) {
    #[inline]
    fn compose_ch(old_ch: char, ch: char, fg: Color) -> char {
        // TODO: change this name like in compose_fg in terms of style
        let preserve_old_char: bool = old_ch != ' ' && fg.a() < 255;
        if preserve_old_char {
            return old_ch;
        }

        ch
    }

    #[inline]
    fn compose_fg(old_fg: Color, fg: Color, old_bg: Color, bg: Color, old_ch: char) -> Color {
        // Case example:
        //
        // White text being covered by a semi-transparent black rect
        // This part would turn the white text darker
        // let bg_is_semi_clear: bool = bg.a() < 255;
        // if bg_is_semi_clear {
        //     return blend_over(old_fg, bg);
        // }

        let old_ch_is_invisible: bool = old_ch == ' ' || old_fg.a() == 0;
        let fg_is_opaque: bool = fg.a() == 255;
        let fg_is_invisible: bool = fg.a() == 0;

        if fg_is_opaque || fg_is_invisible {
            // Override without blending
            return fg;
        }

        if old_ch_is_invisible {
            // Can't blend with old_fg => blend with old_bg instead
            return blend_over(old_bg, fg);
        }

        blend_over(old_fg, fg)

        // if old_ch_is_not_visible {
        //     return blend_over(old_bg, fg);
        // }

        // if !matches!(fg.a(), 0 | 255) {
        //     return blend_over(old_fg, fg);
        // }
    }

    #[inline]
    fn compose_bg(old_bg: Color, bg: Color) -> Color {
        // Preserve old bg if new one is not visible
        if bg.a() == 0 {
            return old_bg;
        }

        let blend_bg_to_bg: bool = !matches!(bg.a(), 0 | 255);
        if blend_bg_to_bg {
            return blend_over(old_bg, bg);
        }

        bg
    }

    // #[inline]
    // fn compose_attributes(old_attributes: Attributes, attributes: Attributes) -> Attributes {}

    let (cols, rows) = (cols as i16, rows as i16);

    for draw_call in draw_calls {
        let mut x: i16 = draw_call.x;
        let y: i16 = draw_call.y;

        // --- Skipping OOB draw calls ---
        let is_oob_left: bool = x < 0;
        let is_oob_top: bool = y < 0;
        let is_oob_right: bool = x >= cols;
        let is_oob_bottom: bool = y >= rows;

        if is_oob_right || is_oob_top || is_oob_bottom {
            continue;
        }

        let mut chars: Chars<'_> = draw_call.rich_text.text.chars();

        // --- Cropping the OOB left side chars ---
        if is_oob_left {
            for _ in 0..(-x) {
                chars.next();
            }
            x = 0;
        }

        let row_start_index: usize = (y as usize) * (cols as usize);
        let remaining_cols: usize = (cols - x).max(0) as usize;

        for (x_offset, ch) in chars.take(remaining_cols).enumerate() {
            let cell_index = row_start_index + x as usize + x_offset;
            let mut buffer_cell: &mut TerminalCell = &mut buffer.0[cell_index];

            let fg: Color = draw_call.rich_text.fg;
            let bg: Color = draw_call.rich_text.bg;
            let attributes: Attributes = draw_call.rich_text.attributes;

            let new_buffer_cell: TerminalCell = TerminalCell {
                ch: compose_ch(buffer_cell.ch, ch, fg),
                fg: compose_fg(buffer_cell.fg, fg, buffer_cell.bg, bg, buffer_cell.ch),
                bg: compose_bg(buffer_cell.bg, bg),
                attributes,
            };

            buffer.0[cell_index] = new_buffer_cell
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
