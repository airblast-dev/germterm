use std::{
    io::{self, Stdout, Write},
    ops::{Deref, DerefMut},
    str::Chars,
};

use crossterm::{cursor as ctcursor, queue, style as ctstyle};

use crate::{
    color::{Color, blend_source_over, lerp},
    rich_text::{Attributes, RichText},
};

pub enum CellFormat {
    Twoxel,
    Octad,
    Standard,
}

pub struct DrawCall {
    pub rich_text: RichText,
    pub x: i16,
    pub y: i16,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub attributes: Attributes,
}

pub struct DiffProduct {
    pub cell: Cell,
    pub x: u16,
    pub y: u16,
}

#[derive(Clone)]
pub struct FrameBuffer(pub Vec<Cell>);

impl Deref for FrameBuffer {
    type Target = [Cell];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FrameBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Frame {
    pub cols: u16,
    pub rows: u16,
    /// Inner Vecs represent layers
    pub draw_queue: Vec<Vec<DrawCall>>,
    /// Used to store post layer flattened `DrawCall`s
    pub flat_draw_queue: Vec<DrawCall>,
    pub current_frame_buffer: FrameBuffer,
    pub old_frame_buffer: FrameBuffer,
    pub diff_products: Vec<DiffProduct>,
}

impl Frame {
    pub fn new(cols: u16, rows: u16) -> Self {
        let vec_capacity: usize = (cols * rows) as usize;
        let empty_buffer: FrameBuffer = FrameBuffer(vec![
            Cell {
                ch: ' ',
                fg: Color::BLACK,
                bg: Color::BLACK,
                attributes: Attributes::NO_FG_COLOR & Attributes::NO_BG_COLOR,
            };
            vec_capacity
        ]);

        Frame {
            cols,
            rows,
            current_frame_buffer: empty_buffer.clone(),
            old_frame_buffer: empty_buffer.clone(),
            diff_products: Vec::with_capacity(vec_capacity),
            // Predefined layer 0
            draw_queue: vec![vec![]],
            flat_draw_queue: Vec::with_capacity(vec_capacity),
        }
    }
}

pub fn compose_frame_buffer(
    buffer: &mut FrameBuffer,
    draw_queue: &[DrawCall],
    cols: u16,
    rows: u16,
    default_blending_color: Color,
) {
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
            let old_cell: Cell = buffer.0[cell_index];
            let new_cell: Cell = Cell {
                ch,
                fg: draw_call.rich_text.fg,
                bg: draw_call.rich_text.bg,
                attributes: draw_call.rich_text.attributes,
            };

            buffer.0[cell_index] = compose_cell(old_cell, new_cell, default_blending_color);
        }
    }
}

pub fn diff_frame_buffers(
    diff_products: &mut Vec<DiffProduct>,
    current_frame_buffer: &FrameBuffer,
    old_frame_buffer: &FrameBuffer,
    cols: u16,
) {
    let cols: usize = cols as usize;

    diff_products.clear();

    let row_pairs = old_frame_buffer
        .chunks(cols)
        .zip(current_frame_buffer.chunks(cols));

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

pub fn build_crossterm_content_style(cell: &Cell) -> crossterm::style::ContentStyle {
    use crossterm::style as ctstyle;

    let no_fg_color: bool = cell.attributes.contains(Attributes::NO_FG_COLOR);
    let no_bg_color: bool = cell.attributes.contains(Attributes::NO_BG_COLOR);

    let fg_color: Option<ctstyle::Color> = if no_fg_color {
        None
    } else {
        Some(ctstyle::Color::Rgb {
            r: cell.fg.r(),
            g: cell.fg.g(),
            b: cell.fg.b(),
        })
    };

    let bg_color: Option<ctstyle::Color> = if no_bg_color {
        None
    } else {
        Some(crossterm::style::Color::Rgb {
            r: cell.bg.r(),
            g: cell.bg.g(),
            b: cell.bg.b(),
        })
    };

    let attributes = [
        (Attributes::BOLD, ctstyle::Attribute::Bold),
        (Attributes::ITALIC, ctstyle::Attribute::Italic),
        (Attributes::UNDERLINED, ctstyle::Attribute::Underlined),
        (Attributes::HIDDEN, ctstyle::Attribute::Hidden),
    ]
    .iter()
    .fold(
        ctstyle::Attributes::none(),
        |crossterm_attrs, (attribute, crossterm_attribute)| {
            if cell.attributes.contains(*attribute) {
                crossterm_attrs | *crossterm_attribute
            } else {
                crossterm_attrs
            }
        },
    );

    ctstyle::ContentStyle {
        foreground_color: fg_color,
        background_color: bg_color,
        underline_color: None,
        attributes,
    }
}

pub(crate) fn draw_to_terminal(
    stdout: &mut Stdout,
    diff_products: &[DiffProduct],
) -> io::Result<()> {
    for diff_product in diff_products.iter() {
        let x: u16 = diff_product.x;
        let y: u16 = diff_product.y;
        let cell: &Cell = &diff_product.cell;

        let style: ctstyle::ContentStyle = build_crossterm_content_style(cell);
        queue!(
            stdout,
            ctcursor::MoveTo(x, y),
            ctstyle::SetAttribute(ctstyle::Attribute::Reset),
            ctstyle::SetStyle(style),
            ctstyle::Print(cell.ch),
        )?;
    }

    stdout.flush()?;
    Ok(())
}

#[inline]
pub(crate) fn copy_frame_buffer(to: &mut FrameBuffer, from: &FrameBuffer) {
    to.copy_from_slice(from);
}

#[inline]
fn compose_cell(old: Cell, new: Cell, default_blending_color: Color) -> Cell {
    // It's important to distinguish the NO_{FG/BG}_COLOR attribute from the Color::CLEAR in usage.
    // They are NOT the same thing and there are differences in how either affects the composition.
    //
    // Why put this metadata in Attributes and not encode the Color to {...}?
    //      A small but real chance of unintentionally hitting that specific value
    // Why not Option<Color>?
    //      Option overhead, this is a very hot loop

    let old_twoxel: bool = old.attributes.contains(Attributes::TWOXEL);
    let old_octad: bool = old.attributes.contains(Attributes::OCTAD);
    let both_ch_equal: bool = old.ch == new.ch;

    // Foreground related
    let new_fg_no_color: bool = new.attributes.contains(Attributes::NO_FG_COLOR);
    let new_fg_invisible: bool = new.fg.a() == 0 || new_fg_no_color;
    let new_fg_opaque: bool = new.fg.a() == 255 || !new_fg_no_color;
    let new_ch_invisible: bool = new.ch == ' ' || new_fg_invisible;

    let old_fg_no_color: bool = old.attributes.contains(Attributes::NO_FG_COLOR);
    let old_ch_blank: bool = old.ch == ' ';
    let old_fg_invisible: bool = old.fg.a() == 0 || old_fg_no_color;
    let old_ch_invisible: bool = new.ch == ' ' || old_fg_invisible;

    // Background related
    let new_bg_no_color: bool = new.attributes.contains(Attributes::NO_BG_COLOR);
    let new_bg_invisible: bool = new.bg.a() == 0 || new_bg_no_color;
    let new_bg_opaque: bool = new.bg.a() == 255 || !new_bg_no_color;
    let new_bg_translucent: bool = !new_bg_opaque && !new_bg_invisible;

    let old_bg_no_color: bool = old.attributes.contains(Attributes::NO_BG_COLOR);
    let old_bg_invisible: bool = old.bg.a() == 0 || old_bg_no_color;

    match cell_format(new.attributes) {
        CellFormat::Standard => {
            let (ch, attributes) = if new_ch_invisible {
                // Covers cases:
                // - Fading an invisible character should not replace the one underneath
                //      - This will keep the old character
                (old.ch, old.attributes)
            } else {
                (new.ch, new.attributes)
            };

            let fg = if new_ch_invisible {
                // Covers cases:
                // - Drawing a translucent bg with no visible char over a visible char
                //      - This will tint the old fg with the new bg to make it appear behind it
                blend_source_over(old.fg, new.bg)
            } else if old_ch_invisible {
                // If there is a char undearneath what we want to draw,
                // the fg should be blended with the old chars to allow
                // for a nicer recovery, especially with fading effects.
                blend_source_over(old.fg, new.fg)
            } else if !old_bg_invisible {
                // Else we try to blend fg with the old bg instead
                blend_source_over(old.bg, new.fg)
            } else {
                Color::RED
            };

            let bg = blend_source_over(old.bg, new.bg);

            Cell {
                ch,
                fg,
                bg,
                attributes,
            }
        }
        // CellFormat::Twoxel => {
        //     let (ch, attributes): (char, Attributes) = if old_twoxel && !new_fg_no_color {
        //         (old.ch, old.attributes)
        //     } else {
        //         (new.ch, new.attributes)
        //     };

        //     let fg: Color = if old_twoxel && both_ch_equal {
        //         blend_source_over(old.fg, new.fg)
        //     } else if old_twoxel {
        //         old.fg
        //     } else if new_fg_opaque {
        //         new.fg
        //     } else if old_fg_invisible || old_ch_blank {
        //         blend_source_over(
        //             if old_bg_no_color {
        //                 default_blending_color
        //             } else {
        //                 old.bg
        //             },
        //             if new_fg_no_color {
        //                 default_blending_color
        //             } else {
        //                 new.fg
        //             },
        //         )
        //     } else {
        //         blend_source_over(old.fg, new.fg)
        //     };

        //     let bg: Color = if old_twoxel && both_ch_equal {
        //         if old_bg_no_color {
        //             default_blending_color
        //         } else {
        //             old.bg
        //         }
        //     } else if old_twoxel && new_bg_no_color {
        //         new.fg
        //     } else if !old_twoxel && old_bg_no_color {
        //         default_blending_color
        //     } else if old_bg_no_color {
        //         new.fg
        //     } else if old_twoxel {
        //         blend_source_over(old.bg, new.fg)
        //     } else {
        //         blend_source_over(old.bg, new.bg)
        //     };

        //     Cell {
        //         ch,
        //         fg,
        //         bg,
        //         attributes,
        //     }
        // }

        // CellFormat::Octad => {
        //     let (ch, attributes): (char, Attributes) = if old_octad && !new_fg_no_color {
        //         (merge_octad(old.ch, new.ch), new.attributes)
        //     } else {
        //         (new.ch, new.attributes)
        //     };

        //     let fg: Color = if new_fg_no_color {
        //         default_blending_color
        //     } else if old_octad {
        //         lerp(
        //             if old_fg_no_color {
        //                 default_blending_color
        //             } else {
        //                 old.fg
        //             },
        //             blend_source_over(old.fg, new.fg),
        //             0.5,
        //         )
        //     } else if old_fg_no_color {
        //         blend_source_over(default_blending_color, new.fg)
        //     } else if new_fg_opaque {
        //         new.fg
        //     } else if new_fg_no_color || old_bg_no_color {
        //         default_blending_color
        //     } else if old_fg_invisible || old_ch_blank {
        //         blend_source_over(old.bg, new.fg)
        //     } else {
        //         blend_source_over(old.fg, new.fg)
        //     };

        //     let bg: Color = if old_bg_no_color && (new_bg_no_color || new_bg_invisible) {
        //         default_blending_color
        //     } else if new_bg_opaque || old_bg_no_color {
        //         new.bg
        //     } else {
        //         blend_source_over(old.bg, new.bg)
        //     };

        //     Cell {
        //         ch,
        //         fg,
        //         bg,
        //         attributes,
        //     }
        // }
        // CellFormat::Standard => {
        //     let (ch, attributes): (char, Attributes) = if new_fg_no_color || new_bg_opaque {
        //         (new.ch, new.attributes)
        //     } else if new_ch_blank {
        //         (old.ch, old.attributes)
        //     } else {
        //         (new.ch, new.attributes)
        //     };

        //     let fg: Color = if new_fg_no_color {
        //         default_blending_color
        //     } else if (new_fg_opaque && !new_ch_blank) || old_fg_no_color {
        //         new.fg
        //     } else if new_bg_translucent {
        //         blend_source_over(old.fg, new.bg)
        //     } else if new_ch_blank {
        //         old.fg
        //     } else if old_fg_invisible || old_ch_blank {
        //         blend_source_over(old.bg, new.fg)
        //     } else {
        //         blend_source_over(old.fg, new.fg)
        //     };

        //     let bg: Color = if old_bg_no_color && (new_bg_no_color || new_bg_invisible) {
        //         default_blending_color
        //     } else if new_bg_opaque || old_bg_no_color {
        //         new.bg
        //     } else if new_bg_no_color {
        //         default_blending_color
        //     } else if new_bg_invisible {
        //         old.bg
        //     } else {
        //         blend_source_over(old.bg, new.bg)
        //     };

        //     Cell {
        //         ch,
        //         fg,
        //         bg,
        //         attributes,
        //     }
        // }
        _ => {
            todo!()
        }
    }
}

#[inline]
fn merge_octad(a: char, b: char) -> char {
    let ma = (a as u32) - 0x2800;
    let mb = (b as u32) - 0x2800;
    std::char::from_u32(0x2800 + (ma | mb)).unwrap()
}

#[inline]
fn cell_format(attrs: Attributes) -> CellFormat {
    if attrs.contains(Attributes::TWOXEL) {
        CellFormat::Twoxel
    } else if attrs.contains(Attributes::OCTAD) {
        CellFormat::Octad
    } else {
        CellFormat::Standard
    }
}
