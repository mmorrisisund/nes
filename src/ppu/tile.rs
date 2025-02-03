use serde::{Deserialize, Serialize};

/// Palettes
///
/// 4 available palettes, 4 colors per palette
///
/// 4bit0
/// -----
/// SAAPP
/// |||||
/// |||++- Pixel value from tile pattern data
/// |++--- Palette number from attributes
/// +----- Background/Sprite select
///

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct BitPlane {
    pub id: u16,
    pub low: u16,
    pub high: u16,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Tile {
    pub next_pattern: BitPlane,
    pub next_palette: u8,

    pattern: BitPlane,
    palette: BitPlane,
}

impl Tile {
    pub fn shift(&mut self) {
        self.palette.low <<= 1;
        self.palette.high <<= 1;

        self.pattern.low <<= 1;
        self.pattern.high <<= 1;
    }

    pub fn load_next_tile(&mut self) {
        self.palette.low |= (self.next_palette as u16 & 0b01) * 0xff;
        self.palette.high |= ((self.next_palette as u16 & 0b10) >> 1) * 0xff;

        self.pattern.low |= self.next_pattern.low;
        self.pattern.high |= self.next_pattern.high;
    }

    pub fn pixel_color(&self, offset: u8) -> (u8, u8) {
        let offset = 15 - offset;

        let color = ((self.pattern.high >> offset) & 0x1) << 1 | (self.pattern.low >> offset) & 0x1;
        let color = if color & 3 > 0 { color } else { 0 };

        let palette =
            ((self.palette.high >> offset) & 0x1) << 1 | (self.palette.low >> offset) & 0x1;

        (palette as u8, color as u8)
    }
}
