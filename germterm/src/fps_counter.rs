//! General FPS metrics.

use crate::engine::Engine;

pub(crate) struct FpsCounter {
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

pub(crate) fn update_fps_counter(fps_counter: &mut FpsCounter, delta_time: f32) {
    if delta_time <= 0.0 {
        return;
    }

    let current_fps: f32 = 1.0 / delta_time;

    if fps_counter.fps_ema <= 0.0 {
        fps_counter.fps_ema = current_fps;
    } else {
        fps_counter.fps_ema = fps_counter.fps_ema * (1.0 - fps_counter.smoothing_factor)
            + current_fps * fps_counter.smoothing_factor;
    }
}

/// Retrieves the current FPS EMA (Exponential Moving Average).
///
/// # Example
/// ```rust,no_run
/// # use germterm::{fps_counter::get_fps, engine::Engine};
/// let mut engine = Engine::new(40, 20);
/// let fps = get_fps(&engine);
/// ```
pub fn get_fps(engine: &Engine) -> f32 {
    engine.fps_counter.fps_ema
}
