pub mod color;
pub mod draw;
pub mod fps_counter;
pub mod fps_limiter;
pub mod input;
pub mod rich_text;
pub mod screen;

use std::io::{self, Write};

use crossterm::{
    cursor, event, execute, queue,
    style::{ContentStyle, Print, SetStyle},
    terminal,
};

use crate::engine::{
    draw::DrawCall,
    fps_counter::{FpsCounter, update_fps_counter},
    fps_limiter::{FpsLimiter, limit_fps, wait_for_next_frame},
    screen::{Screen, build_crossterm_content_style, compose_buffer, diff_buffers},
};

#[derive(Clone, Copy)]
pub struct Pos {
    x: i16,
    y: i16,
}

impl Pos {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy)]
pub struct Size {
    width: i16,
    height: i16,
}

impl Size {
    pub fn new(w: i16, h: i16) -> Self {
        Self {
            width: w,
            height: h,
        }
    }
}

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

    /// A value of `0` will result in uncapped FPS.
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
    update_fps_counter(&mut engine.fps_counter, engine.delta_time);
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

    // !! EXPERIMENTAL BATCHING !!
    // TODO: Test this under actual load, as under no load it cuts FPS in half qq
    // let mut batch_row: Option<u16> = None;
    // let mut batch_style: Option<ContentStyle> = None;
    // let mut batch_start_x: u16 = 0;
    // let mut batch_chars: String = String::new();

    // for (x, y, cell) in engine.screen.buffer_diffs.iter() {
    //     let style: ContentStyle = build_crossterm_content_style(cell);

    //     // (row || style) mismatch => queue batch with crossterm
    //     if batch_row != Some(*y) || batch_style != Some(style) {
    //         if !batch_chars.is_empty() {
    //             queue!(
    //                 engine.stdout,
    //                 cursor::MoveTo(batch_start_x, batch_row.unwrap()),
    //                 Print(&batch_chars)
    //             )?;
    //             batch_chars.clear();
    //         }
    //         batch_start_x = *x;
    //         batch_row = Some(*y);
    //         batch_style = Some(style);
    //         queue!(engine.stdout, SetStyle(style))?;
    //     }

    //     batch_chars.push(cell.ch);
    // }

    // // Queue last batch
    // if !batch_chars.is_empty() {
    //     queue!(
    //         engine.stdout,
    //         cursor::MoveTo(batch_start_x, batch_row.unwrap()),
    //         Print(&batch_chars)
    //     )?;
    // }

    for (x, y, cell) in engine.screen.buffer_diffs.iter() {
        let (x, y) = (*x, *y);
        let style: ContentStyle = build_crossterm_content_style(cell);
        queue!(
            engine.stdout,
            cursor::MoveTo(x, y),
            SetStyle(style),
            Print(cell.ch)
        )?;
    }

    engine.stdout.flush()?;
    engine
        .screen
        .old_buffer
        .0
        .copy_from_slice(&engine.screen.current_buffer.0);
    engine.draw_queue.clear();

    Ok(())
}
