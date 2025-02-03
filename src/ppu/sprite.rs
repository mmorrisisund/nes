use serde::{Deserialize, Serialize};

// TODO: this is shared with bg tile, remove duplication
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct BitPlane {
    // pub address: u16,
    pub low: u8,
    pub high: u8,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Sprite {
    pub pattern: BitPlane,
    pub attribute: u8,
    pub x: u8,
    pub offset: u8,

    pub y: u8,
}

impl Sprite {
    pub fn shift(&mut self) {
        // if self.offset == 0 {
        self.pattern.low <<= 1;
        self.pattern.high <<= 1;
        // } else {
        //     self.offset -= 1;
        // }
    }

    pub fn contains(&self, other_x: u8) -> bool {
        let end = self.x.saturating_add(8);
        (self.x..end).contains(&other_x)
    }

    pub fn pixel_color(&self) -> (u8, u8) {
        let palette = (self.attribute & 0b11) + 4; // sprite palette offset
        let color_lo = (self.pattern.low & 0x80) >> 7;
        let color_hi = (self.pattern.high & 0x80) >> 6;
        let color = color_hi | color_lo;

        if color != 0 {
            (palette, color)
        } else {
            (0, 0)
        }
    }

    pub fn has_priority(&self) -> bool {
        self.attribute & 0x20 == 0x20
    }

    pub fn horizontal_reverse(&mut self) {
        self.pattern.low = self.pattern.low.reverse_bits();
        self.pattern.high = self.pattern.high.reverse_bits();
    }
}
