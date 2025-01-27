// TODO: this is shared with bg tile, remove duplication
#[derive(Debug, Default, Clone, Copy)]
pub struct BitPlane {
    // pub address: u16,
    pub low: u8,
    pub high: u8,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Sprite {
    y: u16,
    tile_lo: u8,
    tile_hi: u8,
    palette: u8,
    has_priority: bool,
    flip_horizontal: bool,
    flip_vertical: bool,

    pub pattern: BitPlane,
    pub attribute: u8,
    pub x: u8,
}

impl Sprite {
    pub fn pixel_color(&self) -> (u8, u8) {
        let palette = (self.attribute & 0b11) + 4; // sprite palette offset
        let pixel_lo = (self.pattern.low & 0x80) >> 7;
        let pixel_hi = (self.pattern.high & 0x80) >> 6;
        let pixel = pixel_hi | pixel_lo;

        if pixel != 0 {
            (pixel, palette)
        } else {
            (0, 0)
        }
    }

    pub fn has_priority(&self) -> bool {
        self.attribute & 0x20 == 0x20
    }
}
