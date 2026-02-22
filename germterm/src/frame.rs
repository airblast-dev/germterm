use crate::{
    cell::{Cell, CellFormat},
    color::{Color, blend_source_over},
    draw::BLOCKTAD_CHAR_LUT,
    layer::Layer,
    rich_text::RichText,
    style::{Attributes, Style},
};
use crossterm::{cursor as ctcursor, queue, style as ctstyle};
use std::{
    io::{self, Stdout, Write},
    ops::{Index, IndexMut},
    str::Chars,
};

#[derive(Clone)]
pub struct DrawCall {
    pub rich_text: RichText,
    pub x: i16,
    pub y: i16,
}

pub struct DiffProduct<'a> {
    pub cell: &'a Cell,
    pub x: u16,
    pub y: u16,
}

pub struct Frame<'a>(&'a [Cell], usize);
pub struct FrameMut<'a>(&'a mut [Cell], usize);
impl<'a> Index<usize> for Frame<'a> {
    type Output = Cell;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index * 2 + self.1]
    }
}

impl<'a> Index<usize> for FrameMut<'a> {
    type Output = Cell;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index * 2 + self.1]
    }
}

impl<'a> IndexMut<usize> for FrameMut<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index * 2 + self.1]
    }
}

#[derive(Clone, Copy, Debug)]
enum FrameOrder {
    CurrentOld = 0,
    OldCurrent = 1,
}

pub struct FramePair {
    /// This stores double of the cell count.
    ///
    /// Each cell is followed by its new or old version depending on the value of [`FrameOrder`]
    pub(crate) frames: Vec<Cell>,
    order: FrameOrder,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) layered_draw_queue: Vec<Layer>,
}

impl FramePair {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            order: FrameOrder::OldCurrent,
            frames: vec![Cell::EMPTY; (width as usize * height as usize) * 2],
            width,
            height,
            layered_draw_queue: vec![],
        }
    }

    pub fn diff(&self) -> impl Iterator<Item = DiffProduct<'_>> {
        debug_assert!(self.frames.len().is_multiple_of(2));
        let width = self.width;
        let order = self.order as usize;

        unsafe { self.frames.as_chunks_unchecked::<2>() }
            .iter()
            .enumerate()
            .filter_map(move |(i, cells @ [left, right])| {
                if left != right {
                    let x = (i % width as usize) as u16;
                    let y = (i / width as usize) as u16;
                    Some(DiffProduct {
                        cell: unsafe { cells.get_unchecked(order) },
                        x,
                        y,
                    })
                } else {
                    None
                }
            })
    }

    pub fn current(&self) -> Frame<'_> {
        Frame(self.frames.as_slice(), self.order as usize)
    }

    pub fn current_mut(&mut self) -> FrameMut<'_> {
        FrameMut(self.frames.as_mut_slice(), self.order as usize)
    }

    /// Swap the current and old frames
    pub fn swap_frames(&mut self) {
        self.order = match self.order {
            FrameOrder::CurrentOld => FrameOrder::OldCurrent,
            FrameOrder::OldCurrent => FrameOrder::CurrentOld,
        };
    }

    pub fn current_mut_and_layered_mut(&mut self) -> (FrameMut<'_>, &mut Vec<Layer>) {
        let frame = FrameMut(&mut self.frames, self.order as usize);
        let layers = &mut self.layered_draw_queue;
        (frame, layers)
    }
}

pub(crate) fn compose_frame_buffer(
    mut buffer: FrameMut<'_>,
    draw_queue: impl Iterator<Item = DrawCall>,
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
            let old_cell: Cell = buffer[cell_index];
            let new_cell: Cell = Cell {
                ch,
                style: draw_call.rich_text.style,
                format: draw_call.rich_text.cell_format,
            };

            buffer[cell_index] = compose_cell(old_cell, new_cell, default_blending_color);
        }
    }
}

pub(crate) fn build_crossterm_content_style(style: Style) -> crossterm::style::ContentStyle {
    use crossterm::style as ctstyle;

    #[inline]
    fn convert_color_to_cs(c: Color) -> ctstyle::Color {
        ctstyle::Color::Rgb {
            r: c.r(),
            g: c.g(),
            b: c.b(),
        }
    }

    let fg_color: Option<ctstyle::Color> = style.fg().map(convert_color_to_cs);
    let bg_color: Option<ctstyle::Color> = style.bg().map(convert_color_to_cs);

    let attributes = [
        (Attributes::BOLD, ctstyle::Attribute::Bold),
        (Attributes::ITALIC, ctstyle::Attribute::Italic),
        (Attributes::UNDERLINED, ctstyle::Attribute::Underlined),
        (Attributes::HIDDEN, ctstyle::Attribute::Hidden),
    ]
    .iter()
    .fold(
        ctstyle::Attributes::none(),
        |ct_attrs, (attribute, ct_attr)| {
            if style.attributes.contains(*attribute) {
                ct_attrs | *ct_attr
            } else {
                ct_attrs
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

pub(crate) fn draw_to_terminal<'a>(
    stdout: &mut Stdout,
    diff_products: impl Iterator<Item = DiffProduct<'a>>,
) -> io::Result<()> {
    for diff_product in diff_products {
        let x: u16 = diff_product.x;
        let y: u16 = diff_product.y;
        let cell: &Cell = diff_product.cell;

        let style: ctstyle::ContentStyle = build_crossterm_content_style(cell.style);
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
fn compose_cell(old: Cell, new: Cell, default_blending_color: Color) -> Cell {
    let both_ch_equal: bool = old.ch == new.ch;

    // Cell format related
    let new_twoxel: bool = new.format == CellFormat::Twoxel;
    let new_octad: bool = new.format == CellFormat::Octad;
    let new_blocktad: bool = new.format == CellFormat::Blocktad;

    let old_twoxel: bool = old.format == CellFormat::Twoxel;
    let old_octad: bool = old.format == CellFormat::Octad;
    let old_blocktad: bool = old.format == CellFormat::Blocktad;

    // Foreground related
    let new_fg = new.style.fg();
    let new_fg_invisible: bool = new_fg.is_some_and(|fg| fg.a() == 0);
    let new_fg_opaque: bool = new_fg.is_some_and(|fg| fg.a() == 0xFF);
    let new_ch_translucent: bool = new.ch != ' ' && !new_fg_invisible && !new_fg_opaque;

    let old_fg = old.style.fg();
    let old_fg_invisible: bool = old_fg.is_some_and(|fg| fg.a() == 0);
    let old_fg_opaque: bool = old_fg.is_some_and(|fg| fg.a() == 255);
    let old_ch_invisible: bool = old.ch == ' ' || old_fg_invisible;

    // Background related
    let new_bg = new.style.bg();
    let new_bg_invisible = new_bg.is_some_and(|bg| bg.a() == 0);
    let new_bg_opaque = new_bg.is_some_and(|bg| bg.a() == 255);
    let new_bg_translucent: bool = !new_bg_opaque && !new_bg_invisible;

    let new_ch_invisible: bool = new.ch == ' ' || new_fg_invisible;

    let old_bg = old.style.bg();
    let old_bg_opaque: bool = old_bg.is_some_and(|c| c.a() == 255);

    let (ch, format, mut attributes, fg, bg) = if new_twoxel {
        let (ch, format, attributes) = if old_twoxel && new_fg.is_some() {
            (old.ch, old.format, old.style.attributes)
        } else {
            (new.ch, new.format, new.style.attributes)
        };

        let fg: Option<Color> = if let Some(nfg) = new_fg {
            if let Some(ofg) = old_fg
                && old_twoxel
                && both_ch_equal
            {
                Some(blend_source_over(ofg, nfg))
            } else if old_twoxel {
                old_fg
            } else if let Some(obg) = old_bg {
                Some(blend_source_over(obg, nfg))
            } else if new_fg_invisible {
                None
            } else {
                Some(blend_source_over(default_blending_color, nfg))
            }
        } else {
            None
        };

        let bg: Option<Color> = if old_twoxel {
            if both_ch_equal {
                old_bg
            } else if old_bg.is_none() {
                if new_fg_invisible {
                    None
                } else {
                    Some(blend_source_over(
                        default_blending_color,
                        new_fg.unwrap_or(Color::CLEAR),
                    ))
                }
            } else {
                Some(blend_source_over(
                    old_bg.unwrap_or(Color::CLEAR),
                    new_fg.unwrap_or(Color::CLEAR),
                ))
            }
        } else {
            old_bg
        };

        (ch, format, attributes, fg, bg)
    } else {
        // This branch handles the following cell formats: [Standard, Octad, Blocktad]
        let (ch, format, attributes) = if new_fg.is_none() && new_bg_opaque && !old_ch_invisible {
            (new.ch, new.format, new.style.attributes)
        } else if new_blocktad && old_blocktad {
            (
                merge_blocktad(old.ch, new.ch),
                new.format,
                new.style.attributes,
            )
        } else if new_octad && old_octad {
            (
                merge_octad(old.ch, new.ch),
                new.format,
                new.style.attributes,
            )
        } else if new_ch_invisible && new_bg.is_some() {
            (old.ch, old.format, old.style.attributes)
        } else {
            (new.ch, new.format, new.style.attributes)
        };

        let fg = if new_ch_invisible && new_bg_opaque {
            None
        } else if new_ch_invisible {
            if new_bg_invisible && old_bg.is_none() {
                old_fg
            } else if new_bg_translucent {
                Some(blend_source_over(
                    old_fg.unwrap_or(Color::CLEAR),
                    new_bg.unwrap_or(Color::CLEAR),
                ))
                .filter(|c| c != &Color::CLEAR)
            } else {
                old_fg
            }
        } else if new_ch_translucent {
            let bottom_color = if !old_ch_invisible {
                old_fg.unwrap_or(Color::CLEAR)
            } else if old_bg.is_none() && new_bg_invisible {
                default_blending_color
            } else if old_bg.is_none() && new_bg_translucent {
                blend_source_over(default_blending_color, new_bg.unwrap_or(Color::CLEAR))
            } else if new_bg_opaque {
                new_bg.unwrap_or(Color::CLEAR)
            } else if old_bg_opaque && new_bg_translucent {
                blend_source_over(
                    old_bg.unwrap_or(Color::CLEAR),
                    new_bg.unwrap_or(Color::CLEAR),
                )
            } else if old_bg_opaque {
                old_bg.unwrap_or(Color::CLEAR)
            } else {
                Color::CLEAR
            };
            Some(blend_source_over(
                bottom_color,
                new_fg.unwrap_or(Color::CLEAR),
            ))
        } else {
            new_fg
        };

        let bg = if new_bg.is_none() || (old_bg.is_none() && new_bg_invisible) {
            None
        } else if new_bg_invisible {
            old_bg
        } else if new_bg_translucent {
            let bottom_color = old_bg.unwrap_or(default_blending_color);
            Some(blend_source_over(
                bottom_color,
                new_bg.unwrap_or(Color::CLEAR),
            ))
        } else {
            new_bg
        };

        (ch, format, attributes, fg, bg)
    };

    // Independent NO_{FG/BG}_COLOR patched into attributes
    attributes = (attributes & !(Attributes::NO_FG_COLOR | Attributes::NO_BG_COLOR))
        | (if fg.is_none() {
            Attributes::NO_FG_COLOR
        } else {
            Attributes::empty()
        })
        | (if bg.is_none() {
            Attributes::NO_BG_COLOR
        } else {
            Attributes::empty()
        });

    Cell {
        ch,
        style: Style::new(fg, bg, attributes),
        format,
    }
}

#[inline]
fn merge_octad(a: char, b: char) -> char {
    let mask_a = (a as u32) - 0x2800;
    let mask_b = (b as u32) - 0x2800;
    std::char::from_u32(0x2800 + (mask_a | mask_b)).unwrap()
}

#[inline]
fn merge_blocktad(a: char, b: char) -> char {
    let mask_a = BLOCKTAD_CHAR_LUT
        .iter()
        .position(|&c| c == a)
        .expect("char not in BLOCKTAD LUT") as u8;
    let mask_b = BLOCKTAD_CHAR_LUT
        .iter()
        .position(|&c| c == b)
        .expect("char not in BLOCKTAD LUT") as u8;

    let merged_mask = mask_a | mask_b;

    BLOCKTAD_CHAR_LUT[merged_mask as usize]
}
