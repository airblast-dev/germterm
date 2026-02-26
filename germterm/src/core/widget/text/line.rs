use crate::{
    core::{
        buffer::slice::SubBuffer,
        draw::{Position, Rect},
        timer::NoDelta,
        widget::{FrameContext, Widget, text::span::Span},
    },
    style::{Stylable, Style},
};

/// A widget that renders a single line composed of one or more [`Span`]s.
///
/// Each span carries its own [`Style`], so a single `Line` can display
/// multiple colors, backgrounds, and text attributes on one row.
///
/// # Lifetimes
///
/// * `'s` — the borrow of the span slice.
/// * `'c` — the lifetime of the text content inside each [`Span`].
#[derive(Debug)]
pub struct Line<'s, Spans: ?Sized = [Span<'s>]> {
    spans: &'s mut Spans,
    style: Style,
}

impl<'s, Spans: ?Sized> Line<'s, Spans>
where
    &'s mut Spans: IntoIterator<Item = &'s mut Span<'s>>,
{
    /// Creates a new `Line` from a mutable slice of [`Span`]s and an
    /// optional base [`Style`].
    pub fn new(spans: &'s mut Spans) -> Self {
        Self {
            spans,
            style: Style::EMPTY,
        }
    }
}

impl Widget<NoDelta> for Line<'_> {
    fn draw(&mut self, ctx: &mut FrameContext<'_, impl crate::core::buffer::Buffer, NoDelta>) {
        let buf = ctx.buffer_mut();
        let sz = buf.size();

        if sz.area() == 0 {
            return;
        }

        let mut offset = 0;
        for span in self.spans.iter_mut() {
            offset = span
                .as_borrowed()
                .with_style(self.style.merged(span.style()))
                .fill_cells(
                    &mut SubBuffer::new(buf, Rect::new(Position::new(offset, 0), sz)),
                    sz.width - offset,
                )
                .saturating_add(offset);
            if offset >= sz.width {
                break;
            }
        }

        for x in offset..sz.width {
            buf.get_cell_mut(Position::new(x, 0))
                .style
                .merge(self.style);
        }
    }
}

impl<Spans: ?Sized> Stylable for Line<'_, Spans> {
    fn style(&self) -> Style {
        self.style
    }

    fn set_style(&mut self, style: Style) {
        self.style = style;
    }
}
