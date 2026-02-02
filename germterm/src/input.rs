//! Input handling.

use crossterm::event::{self, Event};
use std::time::Duration;

/// Polls the terminal for input events and drains all available events.
///
/// This iterator will return every [`crossterm::event::Event`] that is currently available without blocking,
/// ensuring your input handling doesn't fall behind when multiple events accumulate.
///
/// Uses [`crossterm::event::poll`] internally with a zero-duration timeout.
///
/// # Example
/// ```rust,no_run
/// # use germterm::{crossterm::event::Event, input::poll_input};
/// for event in poll_input() {
///     match event {
///         Event::Key(key_event) => println!("Key pressed: {:?}", key_event),
///         Event::Mouse(mouse_event) => println!("Mouse event: {:?}", mouse_event),
///         _ => {}
///     }
/// }
/// ```
pub fn poll_input() -> impl Iterator<Item = Event> {
    std::iter::from_fn(|| {
        if event::poll(Duration::from_millis(0)).ok()? {
            event::read().ok()
        } else {
            None
        }
    })
}
