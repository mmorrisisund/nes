#[derive(Debug, Default, Clone, Copy)]
pub struct Sprite {
    pub x: u16,
    pub y: u16,
    pub tile_lo: u8,
    pub tile_hi: u8,
    pub palette: u8,
    pub has_priority: bool,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

impl Sprite {
    pub fn pixel_color(&self, offset: u8) -> u8 {
        let color = if self.flip_horizontal {
            let lo = (self.tile_lo >> offset) & 0x01;
            let hi = ((self.tile_hi >> offset) & 0x01) << 1;
            hi | lo
        } else {
            let lo = ((self.tile_lo << offset) & 0x80) >> 7;
            let hi = ((self.tile_hi << offset) & 0x80) >> 6;
            hi | lo
        };

        (self.palette << 2) | color
    }
}
