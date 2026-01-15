use crate::{
    Engine,
    color::Color,
    draw::internal::{self},
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
) {
    for state in particle_state.iter_mut() {
        let (x, y) = state.pos;
        internal::draw_braille_dot(draw_calls, x, y, state.color);
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
