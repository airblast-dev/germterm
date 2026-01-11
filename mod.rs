pub mod color;
pub mod draw;
pub mod fps_counter;
pub mod fps_limiter;
pub mod input;
pub mod rich_text;
pub mod screen;

use std::{
    io::{self, Write},
    slice::Chunks,
};

use crossterm::{cursor, event, execute, queue, style::Print, terminal};

use crate::engine::{
    draw::DrawCall,
    fps_counter::FpsCounter,
    fps_limiter::{FpsLimiter, limit_fps, wait_for_next_frame},
    screen::{Screen, TerminalCell, compose_buffer, diff_buffers},
};

pub struct Engine {
    pub delta_time: f32,
    title: &'static str,
    stdout: io::Stdout,
    screen: Screen,
    draw_queue: Vec<DrawCall>,
    fps_limiter: FpsLimiter,
    fps_counter: FpsCounter,
}

impl Engine {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            delta_time: 0.01667,
            title: "my-game",
            stdout: io::stdout(),
            screen: Screen::new(cols, rows),
            draw_queue: Vec::with_capacity((cols * rows) as usize),
            fps_limiter: FpsLimiter::new(60, 0.001, 0.002),
            fps_counter: FpsCounter::new(0.08),
        }
    }

    pub fn title(mut self, value: &'static str) -> Self {
        self.title = value;
        self
    }

    pub fn limit_fps(mut self, value: u32) -> Self {
        limit_fps(&mut self.fps_limiter, value);
        self
    }
}

pub fn init(engine: &mut Engine) -> io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(
        engine.stdout,
        terminal::EnterAlternateScreen,
        terminal::DisableLineWrap,
        terminal::SetTitle("term-slots"),
        event::EnableMouseCapture,
        cursor::Hide,
        terminal::SetSize(engine.screen.cols, engine.screen.rows)
    )?;
    Ok(())
}

pub fn exit_cleanup(engine: &mut Engine) -> io::Result<()> {
    terminal::disable_raw_mode()?;
    execute!(
        engine.stdout,
        terminal::LeaveAlternateScreen,
        cursor::Show,
        event::DisableMouseCapture
    )?;
    Ok(())
}

pub fn start_frame(engine: &mut Engine) {
    engine.delta_time = wait_for_next_frame(&mut engine.fps_limiter);
}

pub fn end_frame(engine: &mut Engine) -> io::Result<()> {
    compose_buffer(
        &mut engine.screen.current_buffer,
        &engine.draw_queue,
        engine.screen.cols,
        engine.screen.rows,
    );
    diff_buffers(
        &mut engine.screen.buffer_diffs,
        &engine.screen.current_buffer,
        &engine.screen.old_buffer,
        engine.screen.cols,
    );

    // TODO: optimize this
    for (x, y, cell) in engine.screen.buffer_diffs.iter() {
        let (x, y) = (*x, *y);
        queue!(engine.stdout, cursor::MoveTo(x, y), Print(cell.ch))?;
    }

    engine.stdout.flush()?;
    engine.draw_queue.clear();

    Ok(())
}
