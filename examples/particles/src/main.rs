use germterm::{
    Engine, Pos,
    color::Color,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind},
    draw::{draw_braille_dot, draw_text, fill_screen},
    end_frame, exit_cleanup,
    fps_counter::draw_fps_counter,
    init,
    input::poll_input,
    start_frame,
};

use rand::Rng;

use std::{f32::consts::PI, io};

pub const TERM_COLS: u16 = 100;
pub const TERM_ROWS: u16 = 50;

struct ParticleState {
    pos: (f32, f32),
    velocity: (f32, f32),
    color: Color,
}

fn main() -> io::Result<()> {
    let mut engine: Engine = Engine::new(TERM_COLS, TERM_ROWS)
        .title("particles")
        .limit_fps(0);

    init(&mut engine)?;

    let mut particles_state: Vec<ParticleState> = Vec::with_capacity(400);

    'game_loop: loop {
        start_frame(&mut engine);
        fill_screen(&mut engine, Color::BLACK);

        for event in poll_input() {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) = event
            {
                break 'game_loop;
            }

            if let Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                kind: KeyEventKind::Press,
                ..
            }) = event
            {
                spawn_particles(&mut particles_state, Color::BLUE, Color::VIOLET);
            }

            if let Event::Key(KeyEvent {
                code: KeyCode::Char('s'),
                kind: KeyEventKind::Press,
                ..
            }) = event
            {
                spawn_particles(&mut particles_state, Color::ORANGE, Color::YELLOW);
            }

            if let Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                kind: KeyEventKind::Press,
                ..
            }) = event
            {
                spawn_particles(&mut particles_state, Color::PINK, Color::CYAN);
            }

            if let Event::Key(KeyEvent {
                code: KeyCode::Char('e'),
                kind: KeyEventKind::Press,
                ..
            }) = event
            {
                spawn_particles(&mut particles_state, Color::PINK, Color::LIME);
            }
        }

        for particle in particles_state.iter_mut() {
            // Friction (your current squared style)
            let friction = 20.0 * engine.delta_time;
            particle.velocity.0 -= particle.velocity.0 * friction.powi(2);
            particle.velocity.1 -= particle.velocity.1 * friction.powi(2);

            // Gravity: constant downward acceleration
            let gravity = 13.8; // tweak this for stronger/weaker effect
            particle.velocity.1 += gravity * engine.delta_time;

            // Update position
            let (x, y) = particle.pos;

            let (dx, dy) = particle.velocity;
            let new_x = x + dx * engine.delta_time * 10.0;
            let new_y = y + dy * engine.delta_time * 10.0;

            particle.pos = (new_x, new_y);

            draw_braille_dot(&mut engine, new_x, new_y, particle.color);
        }

        // Remove OOB bottom particles
        particles_state.retain(|p| p.pos.1 < TERM_ROWS as f32);

        draw_fps_counter(&mut engine, Pos::new(0, 0));

        end_frame(&mut engine)?;
    }

    exit_cleanup(&mut engine)?;
    Ok(())
}

fn spawn_particles(particles_state: &mut Vec<ParticleState>, color_a: Color, color_b: Color) {
    let mut rng = rand::rng();

    let x = TERM_COLS as f32 / 2.0;
    let y = TERM_ROWS as f32 / 3.0;

    let max_speed: f32 = rng.random_range(4.0..16.0);
    let jump_amount: f32 = rng.random_range(2.0..8.0);

    for _ in 0..rng.random_range(100..1000) {
        let angle = rng.random_range(0.0..2.0 * PI);
        let speed = rng.random_range(0.5..max_speed);

        let vx = speed * angle.cos();
        let vy = (speed * angle.sin() - jump_amount) * 0.5;

        let color = if rng.random_bool(0.5) {
            color_a.with_alpha(127)
        } else {
            color_b.with_alpha(127)
        };

        // let color = Color::new(
        //     rng.random_range(0..=255),
        //     rng.random_range(0..=255),
        //     rng.random_range(0..=255),
        //     255,
        // );

        particles_state.push(ParticleState {
            pos: (x, y),
            velocity: (vx, vy),
            color,
        });
    }
}
