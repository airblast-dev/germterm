use crate::engine::{Engine, color::Color, rich_text::RichText};

pub struct DrawCall {
    pub rich_text: RichText,
    pub x: i16,
    pub y: i16,
}

pub fn draw_text(engine: &mut Engine, x: i16, y: i16, text: impl Into<RichText>) {
    let rich_text: RichText = text.into();
    engine.draw_queue.push(DrawCall { rich_text, x, y });
}
