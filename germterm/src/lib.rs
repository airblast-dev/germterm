#![doc = include_str!("./../README.md")]

pub use crossterm;

pub mod color;
pub mod draw;
pub mod engine;
pub mod fps_counter;
mod fps_limiter;
mod frame;
pub mod input;
mod metadata;
pub mod particle;
pub mod rich_text;
