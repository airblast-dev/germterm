use crate::{Engine, color::Color, rich_text::RichText};

#[derive(Clone, Copy)]
pub struct Pos {
    x: i16,
    y: i16,
}

impl Pos {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    /// Scales the `x` argument by 2 to compensate for typical terminal cell aspect ratios,
    /// making drawn shapes appear closer to a square.
    pub fn square(x: i16, y: i16) -> Self {
        let x: i16 = x * 2;
        Self { x, y }
    }
}

#[derive(Clone, Copy)]
pub struct Size {
    width: i16,
    height: i16,
}

impl Size {
    pub fn new(w: i16, h: i16) -> Self {
        Self {
            width: w,
            height: h,
        }
    }

    /// Scales the `width` argument by 2 to compensate for typical terminal cell aspect ratios,
    /// making drawn shapes appear closer to a square.
    pub fn square(width: i16, height: i16) -> Self {
        let width: i16 = width * 2;
        Self { width, height }
    }
}

pub fn fill_screen(engine: &mut Engine, color: Color) {
    let cols: i16 = engine.frame.cols as i16;
    let rows: i16 = engine.frame.rows as i16;
    internal::fill_screen(&mut engine.frame.draw_queue, cols, rows, color);
}

pub fn draw_text(engine: &mut Engine, pos: Pos, text: impl Into<RichText>) {
    internal::draw_text(&mut engine.frame.draw_queue, pos, text);
}

pub fn draw_rect(engine: &mut Engine, pos: Pos, size: Size, color: Color) {
    internal::draw_rect(&mut engine.frame.draw_queue, pos, size, color);
}

pub fn draw_braille_dot(engine: &mut Engine, x: f32, y: f32, color: Color) {
    internal::draw_braille_dot(&mut engine.frame.draw_queue, x, y, color);
}

pub(crate) mod internal {
    use crate::{
        Engine,
        color::Color,
        draw::{Pos, Size},
        frame::DrawCall,
        rich_text::RichText,
    };

    pub fn fill_screen(draw_queue: &mut Vec<DrawCall>, cols: i16, rows: i16, color: Color) {
        draw_rect(draw_queue, Pos::new(0, 0), Size::new(cols, rows), color);
    }

    pub fn draw_text(draw_queue: &mut Vec<DrawCall>, pos: Pos, text: impl Into<RichText>) {
        let rich_text: RichText = text.into();
        draw_queue.push(DrawCall {
            rich_text,
            x: pos.x,
            y: pos.y,
        });
    }

    pub fn draw_rect(draw_queue: &mut Vec<DrawCall>, pos: Pos, size: Size, color: Color) {
        let row_text: String = " ".repeat(size.width as usize);
        let row_rich_text: RichText = RichText::new(&row_text).fg(Color::BLACK).bg(color);

        for row in 0..size.height {
            draw_text(
                draw_queue,
                Pos::new(pos.x, pos.y + row),
                row_rich_text.clone(),
            )
        }
    }

    pub fn draw_braille_dot(draw_queue: &mut Vec<DrawCall>, x: f32, y: f32, color: Color) {
        let cell_x: i16 = x.floor() as i16;
        let cell_y: i16 = y.floor() as i16;
        let cell_pos: Pos = Pos::new(cell_x, cell_y);

        let sub_x: u8 = ((x - cell_x as f32) * 2.0).clamp(0.0, 1.0) as u8;
        let sub_y_float: f32 = (y - cell_y as f32) * 4.0;
        let sub_y: usize = sub_y_float.floor().clamp(0.0, 3.0) as usize;

        let offset: usize = match (sub_x, sub_y) {
            (0, 0) => 0,
            (0, 1) => 1,
            (0, 2) => 2,
            (0, 3) => 6,
            (1, 0) => 3,
            (1, 1) => 4,
            (1, 2) => 5,
            (1, 3) => 7,
            _ => 0,
        };

        let braille_char: char = std::char::from_u32(0x2800 + (1 << offset)).unwrap();
        let rich_text: RichText = RichText::new(braille_char.to_string()).fg(color);

        draw_text(draw_queue, cell_pos, rich_text);
    }
}
