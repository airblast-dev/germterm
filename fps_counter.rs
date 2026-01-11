use crate::engine::{
    Engine,
    draw::{DrawCall, draw_text},
    rich_text::RichText,
};

pub struct FpsCounter {
    fps_ema: f32,
    smoothing_factor: f32,
}

impl FpsCounter {
    pub fn new(smoothing_factor: f32) -> Self {
        Self {
            fps_ema: 0.0,
            smoothing_factor,
        }
    }
}

pub fn update_fps_counter(fps_counter: &mut FpsCounter, dt: f32) {
    if dt <= 0.0 {
        return;
    }

    let current_fps: f32 = 1.0 / dt;

    if fps_counter.fps_ema <= 0.0 {
        fps_counter.fps_ema = current_fps;
    } else {
        fps_counter.fps_ema = fps_counter.fps_ema * (1.0 - fps_counter.smoothing_factor)
            + current_fps * fps_counter.smoothing_factor;
    }
}

pub fn draw_fps_counter(engine: &mut Engine, x: i16, y: i16) {
    draw_text(
        engine,
        x,
        y,
        format!("FPS: {:2.0}", engine.fps_counter.fps_ema),
    );
}
