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
