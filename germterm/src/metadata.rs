#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellFormat {
    Standard = 0b_0000,
    Twoxel = 0b0001,
    Octad = 0b0010,
    Blocktad = 0b0011,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Metadata(u8);

impl Metadata {
    const EXCLUSIVE_FORMAT_MASK: u8 = 0b0000_1111;
    const NO_FG_COLOR_MASK: u8 = 0b0001_0000;
    const NO_BG_COLOR_MASK: u8 = 0b0010_0000;

    #[inline]
    pub fn cell_format(self) -> CellFormat {
        match self.0 & Self::EXCLUSIVE_FORMAT_MASK {
            0b0000 => CellFormat::Standard,
            0b0001 => CellFormat::Twoxel,
            0b0010 => CellFormat::Octad,
            0b0011 => CellFormat::Blocktad,
            _ => panic!("Unexpected cell format"),
        }
    }

    #[inline]
    pub const fn empty() -> Self {
        Metadata(0b_0000_0000)
    }

    #[inline]
    pub fn with_cell_format(mut self, format: CellFormat) -> Self {
        self.0 = (self.0 & !Self::EXCLUSIVE_FORMAT_MASK) | (format as u8);
        self
    }

    #[inline]
    pub fn with_no_fg_color(mut self, enabled: bool) -> Self {
        if enabled {
            self.0 |= Self::NO_FG_COLOR_MASK;
        } else {
            self.0 &= !Self::NO_FG_COLOR_MASK;
        }
        self
    }

    #[inline]
    pub fn with_no_bg_color(mut self, enabled: bool) -> Self {
        if enabled {
            self.0 |= Self::NO_BG_COLOR_MASK;
        } else {
            self.0 &= !Self::NO_BG_COLOR_MASK;
        }
        self
    }

    #[inline]
    pub fn no_fg_color(&self) -> bool {
        (self.0 & Self::NO_FG_COLOR_MASK) != 0
    }

    #[inline]
    pub fn no_bg_color(&self) -> bool {
        (self.0 & Self::NO_BG_COLOR_MASK) != 0
    }
}
