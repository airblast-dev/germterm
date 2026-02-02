//! Octad-based particle system.
//!
//! This module provides a way of spawning particles using the [`spawn_particles`] function.
//! Particles are automatically updated and drawn at the end of the frame.
//!
//! The particles and their behaviors can be customized using [`ParticleSpec`] and [`ParticleEmitter`].
//! The system uses approximated velocity, gravity and drag calculations.
//!
//! ## Notes
//! Particles are always drawn at the end of the frame. This means they'll always be drawn last on the specified layer.
//! If you wish to spawn particles underneath other drawn elements, you can create a new layer with a lower index and draw to it.

use std::{f32::consts::PI, ops::RangeInclusive};

use rand::{Rng, rngs::ThreadRng};

use crate::{
    color::{Color, ColorGradient, sample_gradient},
    draw::{
        Layer,
        internal::{self},
    },
    engine::Engine,
    frame::DrawCall,
};

pub enum ParticleEmitterShape {
    Circle,
    Cone { direction_deg: f32, width_deg: f32 },
}

#[derive(Clone)]
pub enum ParticleColor {
    Solid(Color),
    Gradient(ColorGradient),
}

pub(crate) struct ParticleState {
    pos: (f32, f32),
    velocity: (f32, f32),
    color: ParticleColor,
    gravity_scale: f32,
    spawn_timestamp: f32,
    death_timestamp: f32,
    layer: Layer,
}

pub struct ParticleSpec {
    // TODO: Make this also support a weighted set of colors
    pub color: ParticleColor,
    pub speed: RangeInclusive<f32>,
    pub lifetime_sec: f32,
    pub gravity_scale: f32,
}

impl Default for ParticleSpec {
    fn default() -> Self {
        Self {
            color: ParticleColor::Solid(Color::WHITE),
            speed: 15.0..=30.0,
            lifetime_sec: 3.0,
            gravity_scale: 1.0,
        }
    }
}

pub struct ParticleEmitter {
    pub shape: ParticleEmitterShape,
    pub count: usize,
}

impl Default for ParticleEmitter {
    fn default() -> Self {
        Self {
            shape: ParticleEmitterShape::Circle,
            count: 25,
        }
    }
}

/// Spawns particles once at a position with specified parameters.
///
/// Particles can be customized by tinkering with the `spec` and `emitter` parameters.
///
/// # Examples
/// ```rust,no_run
/// # use germterm::{draw::Layer, engine::Engine, particle::{spawn_particles, ParticleSpec, ParticleEmitter}};
/// let mut engine = Engine::new(40, 20);
/// let mut layer = Layer::new(&mut engine, 0);
///
/// let spec = ParticleSpec::default();
/// let emitter = ParticleEmitter::default();
/// spawn_particles(&mut layer, 20.0, 10.0, &spec, &emitter);
/// ```
pub fn spawn_particles(
    layer: &mut Layer,
    x: f32,
    y: f32,
    spec: &ParticleSpec,
    emitter: &ParticleEmitter,
) {
    let mut rng: ThreadRng = rand::rng();
    let engine_mut: &mut Engine = unsafe { &mut *layer.engine_ptr };

    match emitter.shape {
        ParticleEmitterShape::Circle => {
            for _ in 0..emitter.count {
                let angle: f32 = rng.random_range(0.0..=2.0 * PI);
                let speed: f32 = rng.random_range(spec.speed.clone());

                let velocity_x: f32 = speed * angle.cos();
                let velocity_y: f32 = speed * angle.sin();

                engine_mut.particle_state.push(ParticleState {
                    pos: (x, y),
                    velocity: (velocity_x, velocity_y),
                    color: spec.color.clone(),
                    gravity_scale: spec.gravity_scale,
                    spawn_timestamp: engine_mut.game_time,
                    death_timestamp: engine_mut.game_time + spec.lifetime_sec,
                    layer: *layer,
                })
            }
        }
        ParticleEmitterShape::Cone {
            direction_deg,
            width_deg,
        } => {
            for _ in 0..emitter.count {
                let half_angle_rad: f32 = (width_deg / 2.0).to_radians(); // ← Use spread_deg as width
                let direction_rad: f32 = direction_deg.to_radians();

                let random_angle_offset: f32 = rng.random_range(-half_angle_rad..half_angle_rad);
                let particle_angle: f32 = direction_rad + random_angle_offset;

                let speed: f32 = rng.random_range(spec.speed.clone());
                let velocity_x: f32 = speed * particle_angle.cos();
                let velocity_y: f32 = speed * particle_angle.sin();

                engine_mut.particle_state.push(ParticleState {
                    pos: (x, y),
                    velocity: (velocity_x, velocity_y),
                    color: spec.color.clone(),
                    gravity_scale: spec.gravity_scale,
                    spawn_timestamp: engine_mut.game_time,
                    death_timestamp: engine_mut.game_time + spec.lifetime_sec,
                    layer: *layer,
                })
            }
        }
    }
}

/// Tiny debug helper that displays the alive particle count.
#[inline]
pub fn particle_count(engine: &Engine) -> usize {
    engine.particle_state.len()
}

pub(crate) fn update_and_draw_particles(engine: &mut Engine) {
    let gravity: f32 = 200.0;
    let drag: f32 = 3.0;
    let drag_decay: f32 = 1.0 / (1.0 + drag * engine.delta_time);
    // y:x aspect ratio to account for terminal cells not being perfect squares
    // and not making the end result look stretched out vertically
    let aspect_ratio: f32 = 1.0 / 2.0;

    let mut i: usize = 0;
    while i < engine.particle_state.len() {
        let state: &mut ParticleState = &mut engine.particle_state[i];

        if engine.game_time >= state.death_timestamp {
            engine.particle_state.swap_remove(i);
            continue;
        }

        let t: f32 = ((engine.game_time - state.spawn_timestamp)
            / (state.death_timestamp - state.spawn_timestamp))
            .clamp(0.0, 1.0);

        let color: Color = match &state.color {
            ParticleColor::Solid(color) => *color,
            ParticleColor::Gradient(color_gradient) => sample_gradient(color_gradient, t),
        };

        state.velocity.1 += gravity * state.gravity_scale * engine.delta_time;

        state.velocity.0 *= drag_decay;
        state.velocity.1 *= drag_decay;

        state.pos.0 += state.velocity.0 * engine.delta_time;
        state.pos.1 += state.velocity.1 * engine.delta_time * aspect_ratio;

        let draw_queue: &mut Vec<DrawCall> =
            &mut engine.frame.layered_draw_queue[state.layer.index];
        internal::draw_octad(draw_queue, state.pos.0, state.pos.1, color);

        i += 1;
    }
}
