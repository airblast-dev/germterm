/// A packed RGBA color stored as a single `u32`.
///
/// Layout:
/// `0xRR_GG_BB_AA`
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

    pub fn r(&self) -> u8 {
        ((self.0 >> 24) & 0xFF) as u8
    }

    pub fn g(&self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    pub fn b(&self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    pub fn a(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    pub fn rgb(&self) -> (u8, u8, u8) {
        (self.r(), self.g(), self.b())
    }

    pub fn rgba(&self) -> (u8, u8, u8, u8) {
        (self.r(), self.g(), self.b(), self.a())
    }

    pub fn rgba_f32(&self) -> (f32, f32, f32, f32) {
        let r: f32 = ((self.0 >> 24) & 0xFF) as f32 / 255.0;
        let g: f32 = ((self.0 >> 16) & 0xFF) as f32 / 255.0;
        let b: f32 = ((self.0 >> 8) & 0xFF) as f32 / 255.0;
        let a: f32 = (self.0 & 0xFF) as f32 / 255.0;
        (r, g, b, a)
    }

    pub fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color::new(
            (r.clamp(0.0, 1.0) * 255.0) as u8,
            (g.clamp(0.0, 1.0) * 255.0) as u8,
            (b.clamp(0.0, 1.0) * 255.0) as u8,
            (a.clamp(0.0, 1.0) * 255.0) as u8,
        )
    }
}

pub fn blend_over(bottom: Color, top: Color) -> Color {
    let (br, bg, bb, ba) = bottom.rgba_f32();
    let (tr, tg, tb, ta) = top.rgba_f32();

    let out_a = ta + ba * (1.0 - ta);
    if out_a <= 0.0 {
        return Color::CLEAR;
    }

    let out_r = (tr * ta + br * ba * (1.0 - ta)) / out_a;
    let out_g = (tg * ta + bg * ba * (1.0 - ta)) / out_a;
    let out_b = (tb * ta + bb * ba * (1.0 - ta)) / out_a;

    Color::from_f32(out_r, out_g, out_b, out_a)
}
