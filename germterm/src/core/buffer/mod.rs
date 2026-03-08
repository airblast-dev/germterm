pub mod diffed;
pub mod flat;
pub mod paired;
pub mod slice;
pub mod test;
pub mod utils;

use super::DrawCall;
use crate::{
    cell::Cell,
    core::{Position, draw::Size},
};

/// Indicates which axis (or axes) caused an out-of-bounds access.
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorOutOfBoundsAxises {
    /// The X coordinate was out of bounds.
    X,
    /// The Y coordinate was out of bounds.
    Y,
    /// Both the X and Y coordinates were out of bounds.
    XY,
}

/// A 2D grid of [`Cell`]s that can be read and written by position.
///
/// Implementors manage their own internal storage of [`Cell`]s. This trait
/// provides a unified way for widgets to write and read from a [`Buffer`].
///
/// # Checked vs unchecked methods
///
/// This trait provides two variants of read/write methods:
/// - **Checked** methods (`*_checked`) return a [`Result`] that indicates whether
///   the position was in bounds. Use these when the position may be invalid.
/// - **Unchecked** methods panic if the position is out of bounds. Use these
///   when you are certain the position is valid, such as when iterating over
///   [`Buffer::size`].
///
/// # Frame lifecycle
///
/// Buffers are typically used in a frame-based rendering cycle:
/// 1. [`start_frame`] is called at the beginning of a frame to prepare the buffer
/// 2. Widgets write cells to the buffer
/// 3. [`end_frame`] is called to finalize the frame
///
/// The default implementations of [`end_frame`] calls [`Buffer::flush`] and for [`start_frame`] it clears the
/// buffer via [`Buffer::clear`].
///
/// For buffers that support resizing, see [`ResizableBuffer`].
///
/// [`start_frame`]: Buffer::start_frame
/// [`end_frame`]: Buffer::end_frame
pub trait Buffer {
    /// The size of the area that can be drawn in this buffer
    fn size(&self) -> Size;

    /// Sets the cell at `pos`, returning an error if `pos` is outside bounds.
    fn set_cell_checked(&mut self, pos: Position, cell: Cell)
    -> Result<(), ErrorOutOfBoundsAxises>;
    /// Sets the cell at `pos` without bounds checking.
    ///
    /// # Panics
    ///
    /// Panics if `pos` is out of bounds.
    fn set_cell(&mut self, pos: Position, cell: Cell) {
        self.set_cell_checked(pos, cell)
            .expect("out of bounds set_cell")
    }

    /// Returns a reference to the cell at `pos`, returning an error if `pos` is outside bounds.
    fn get_cell_checked(&self, pos: Position) -> Result<&Cell, ErrorOutOfBoundsAxises>;

    /// Returns a reference to the cell at `pos` without bounds checking.
    ///
    /// # Panics
    ///
    /// Panics if `pos` is out of bounds.
    fn get_cell(&self, pos: Position) -> &Cell {
        self.get_cell_checked(pos).expect("out of bounds get_cell")
    }

    /// Returns a mutable reference to the cell at `pos`, returning an error if `pos` is outside bounds.
    fn get_cell_mut_checked(&mut self, pos: Position) -> Result<&mut Cell, ErrorOutOfBoundsAxises>;

    /// Returns a mutable reference to the cell at `pos` without bounds checking.
    ///
    /// # Panics
    ///
    /// Panics if `pos` is out of bounds.
    #[track_caller]
    fn get_cell_mut(&mut self, pos: Position) -> &mut Cell {
        let sz = self.size();
        self.get_cell_mut_checked(pos).unwrap_or_else(|_| {
            panic!(
                "out of bounds get_cell_mut for position: {:?} with size: {:?}",
                pos, sz
            )
        })
    }

    /// Fills the entire buffer with `cell`.
    fn fill(&mut self, cell: Cell) {
        let size = self.size();
        for y in 0..size.height {
            for x in 0..size.width {
                self.set_cell(Position { x, y }, cell);
            }
        }
    }

    /// Clears the buffer by filling it with [`Cell::EMPTY`].
    fn clear(&mut self) {
        self.fill(Cell::EMPTY);
    }

    /// Called at the beginning of a frame.
    ///
    /// Implementations may override this to prepare the buffer for new draw
    /// commands. The default implementation clears the cells.
    fn start_frame(&mut self) {
        self.clear();
    }

    /// Called at the end of a frame.
    ///
    /// Implementations may override this to finalize the buffer contents before
    /// rendering. The default implementation calls [`flush`](Buffer::flush).
    fn end_frame(&mut self) {
        self.flush();
    }

    /// Flushes any pending state.
    ///
    /// This is called by the default [`end_frame`](Buffer::end_frame)
    /// implementation. Override this if your buffer needs to perform cleanup
    /// or synchronization at the end of a frame.
    ///
    /// If calling widget with a manually created temporary buffer, this method should be called before
    /// reading from it.
    fn flush(&mut self) {}
}

/// A [`Buffer`] that can be resized at runtime.
///
/// This trait extends [`Buffer`] with the ability to change dimensions.
/// After calling [`resize`], the buffer's [`Buffer::size`] method must
/// return the new size.
///
/// [`resize`]: ResizableBuffer::resize
pub trait ResizableBuffer: Buffer {
    /// Resizes this buffer to `size`.
    ///
    /// After this call, [`Buffer::size`] must return the provided size.
    /// Existing cell contents may or may not be preserved depending on the
    /// implementation.
    fn resize(&mut self, size: Size);
}

/// Produces an iterator of [`DrawCall`]s representing cells that need to be
/// rendered to the terminal for the current frame.
pub trait Drawer {
    fn draw(&mut self) -> impl Iterator<Item = DrawCall<'_>>;
}
