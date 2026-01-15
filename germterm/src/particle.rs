use crate::{
    color::Color,
    draw::internal::{self},
    engine::Engine,
    frame::DrawCall,
};

pub struct ParticleState {
    pos: (f32, f32),
    velocity: (f32, f32),
    color: Color,
    // spawn_timestamp: f32,
    // death_timestamp: f32,
}

pub struct ParticleSpec {
    // TODO: Make this also support a weighted set of colors and possibly gradients
    color: Color,
    speed: f32,
    // lifetime_sec: f32,
}

impl ParticleSpec {
    pub fn new(color: Color) -> Self {
        Self { color, speed: 30.0 }
    }

    pub fn with_speed(mut self, value: f32) -> Self {
        self.speed = value;
        self
    }
}

pub struct ParticleEmitter {
    count: usize,
}

impl ParticleEmitter {
    pub fn new() -> Self {
        Self { count: 1 }
    }
}

pub(crate) fn update_and_draw_particles(
    particle_state: &mut [ParticleState],
    draw_calls: &mut Vec<DrawCall>,
    delta_time: f32,
) {
    let drag_factor: f32 = 20.0 * delta_time;
    let gravity: f32 = 13.8;

    for state in particle_state.iter_mut() {
        let (x, y) = state.pos;

        let (mut velocity_x, mut velocity_y) = state.velocity;

        velocity_x -= velocity_x * drag_factor.powi(2);
        velocity_y -= velocity_y * drag_factor.powi(2);

        velocity_y += gravity * delta_time;

        let new_x: f32 = x + velocity_x * delta_time * 10.0;
        let new_y: f32 = y + velocity_y * delta_time * 10.0;
        state.pos = (new_x, new_y);

        internal::draw_braille_dot(draw_calls, new_x, new_y, state.color);
    }
}

pub fn spawn_particles(
    engine: &mut Engine,
    x: f32,
    y: f32,
    spec: &ParticleSpec,
    emitter: &ParticleEmitter,
) {
    for _ in 0..emitter.count {
        // Velocity (0, 0) for now
        let velocity_x: f32 = 0.0;
        let velocity_y: f32 = 0.0;

        engine.particle_state.push(ParticleState {
            pos: (x, y),
            velocity: (velocity_x, velocity_y),
            color: spec.color,
        })
    }
}
