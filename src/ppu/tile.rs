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

#[derive(Debug, Default, Clone, Copy)]
pub struct Pattern {
    pub low: u8,
    pub high: u8,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Tile {
    pub pattern_address: u16,  // address of next pattern
    pub next_palette: u8,      // data from attribute table
    pub next_pattern: Pattern, // next pattern
    palette_hi: u16,           // current palette
    palette_lo: u16,           // current palette
    pattern_hi: u16,           // high bit plane of pattern data
    pattern_lo: u16,           // low bit plane of pattern data
}

impl Tile {
    pub fn shift(&mut self) {
        self.palette_lo <<= 1;
        self.palette_hi <<= 1;
        self.pattern_lo <<= 1;
        self.pattern_hi <<= 1;
    }

    pub fn load_next_tile(&mut self) {
        self.palette_hi |= ((self.next_palette & 0b10) * 0xff) as u16;
        self.palette_lo |= ((self.next_palette & 0b01) * 0xff) as u16;

        self.pattern_hi |= self.next_pattern.high as u16;
        self.pattern_lo |= self.next_pattern.low as u16;
    }

    pub fn pixel_color(&self, offset: u8) -> u8 {
        let offset = offset as u16;
        let color_lo = ((self.pattern_lo << offset) & 0x8000) >> 15;
        let color_hi = ((self.pattern_hi << offset) & 0x8000) >> 14;
        let color = color_hi | color_lo;

        let palette_lo = (self.palette_lo << offset) << 2;
        let palette_hi = (self.palette_hi << offset) << 3;
        let palette = palette_hi | palette_lo;

        (palette | color) as u8
    }
}
