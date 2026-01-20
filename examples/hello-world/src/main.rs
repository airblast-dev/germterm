use std::io;

use germterm::{
    color::Color,
    crossterm::event::{Event, KeyCode, KeyEvent},
    draw::{draw_text, fill_screen},
    engine::{Engine, end_frame, exit_cleanup, init, start_frame},
    fps_counter::draw_fps_counter,
    input::poll_input,
};

fn main() -> io::Result<()> {
    let mut engine: Engine = Engine::new(40, 20).limit_fps(60);

    init(&mut engine)?;

    'update_loop: loop {
        for event in poll_input() {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) = event
            {
                break 'update_loop;
            }
        }

        start_frame(&mut engine);

        fill_screen(&mut engine, Color::BLACK);
        draw_text(&mut engine, 14, 9, "Hello world!");
        draw_fps_counter(&mut engine, 0, 0);

        end_frame(&mut engine)?;
    }

    exit_cleanup(&mut engine)?;
    Ok(())
}
