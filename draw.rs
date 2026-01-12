use crate::engine::{Engine, Pos, Size, color::Color, rich_text::RichText};

pub struct DrawCall {
    pub rich_text: RichText,
    pub x: i16,
    pub y: i16,
}

pub fn draw_text(engine: &mut Engine, pos: Pos, text: impl Into<RichText>) {
    let rich_text: RichText = text.into();
    engine.draw_queue.push(DrawCall {
        rich_text,
        x: pos.x,
        y: pos.y,
    });
}

pub fn draw_rect(engine: &mut Engine, pos: Pos, size: Size, color: Color) {
    let row_text: String = " ".repeat(size.width as usize);
    let row_rich_text: RichText = RichText::new(&row_text).fg(Color::CLEAR).bg(color);

    for row in 0..size.height {
        draw_text(engine, Pos::new(pos.x, pos.y + row), row_rich_text.clone())
    }
}

pub fn draw_fps_counter(engine: &mut Engine, pos: Pos) {
    draw_text(
        engine,
        pos,
        format!("FPS: {:2.0}", engine.fps_counter.fps_ema),
    );
}

pub fn fill_screen(engine: &mut Engine, color: Color) {
    let cols: i16 = engine.screen.cols as i16;
    let rows: i16 = engine.screen.rows as i16;
    draw_rect(engine, Pos::new(0, 0), Size::new(cols, rows), color);
}
