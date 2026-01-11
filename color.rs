/// A packed RGBA color stored as a single `u32`.
///
/// Layout:
/// `RR GG BB AA`
///
/// # Examples
///
/// ```
/// let color = Color::new(255, 0, 0, 255);
/// assert_eq!(color, Color::RED);
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Color(u32);

impl Color {
    pub const WHITE: Self = Color(0xFF_FF_FF_FF);
    pub const BLACK: Self = Color(0x00_00_00_FF);
    pub const RED: Color = Color(0xFF_00_00_FF);
    pub const GREEN: Color = Color(0x00_FF_00_FF);
    pub const BLUE: Color = Color(0x00_00_FF_FF);
    pub const CLEAR: Color = Color(0x00_00_00_00);

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32))
    }
}
