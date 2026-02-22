//! Stylized text.

use bitflags::Flags;

use crate::{
    cell::CellFormat,
    color::Color,
    style::{Attributes, Style},
};
use std::sync::Arc;

/// Stylized text representation.
///
/// Bundles together text, foreground color, background color and attributes.
///
/// # Conversions
/// `RichText` can be created from the following types:
/// - `String`
/// - `&str`
#[derive(Clone)]
pub struct RichText {
    pub text: Arc<String>,
    pub style: Style,
    pub(crate) cell_format: CellFormat,
}

impl RichText {
    /// Creates a new `RichText` with default styling.
    ///
    /// To customize the style, use the following builder methods:
    /// - [`RichText::withg_fg()`]
    /// - [`RichText::with_bg()`]
    /// - [`RichText::with_attributes()`]
    ///
    /// `&str` and `String` types can be turned `into()`, which are converted into [`RichText`].
    #[inline]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: Arc::new(text.into()),
            style: Style::new(None, None, Attributes::empty()),
            cell_format: CellFormat::Standard,
        }
    }

    #[inline]
    pub fn with_fg(mut self, color: Option<Color>) -> Self {
        self.style.set_fg(color);
        self
    }

    #[inline]
    pub fn with_bg(mut self, color: Option<Color>) -> Self {
        self.style.set_bg(color);
        self
    }

    #[inline]
    pub(crate) fn with_cell_format(mut self, format: CellFormat) -> Self {
        self.cell_format = format;
        self
    }

    #[inline]
    pub fn with_attributes(mut self, attrs: Attributes) -> Self {
        self.style.attributes |= attrs
            & (Attributes::ITALIC | Attributes::BOLD | Attributes::HIDDEN | Attributes::UNDERLINED);

        self
    }
}

impl From<String> for RichText {
    #[inline]
    fn from(s: String) -> Self {
        RichText::new(s)
    }
}

impl<'a> From<&'a str> for RichText {
    #[inline]
    fn from(s: &'a str) -> Self {
        RichText::new(s)
    }
}
