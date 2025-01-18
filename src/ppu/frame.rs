use super::TOTAL_PIXELS;

#[derive(Debug)]
pub struct Frame {
    buffer: [u8; TOTAL_PIXELS],
}

impl Frame {
    pub fn new() -> Self {
        Self {
            buffer: [0; TOTAL_PIXELS],
        }
    }
    pub fn set_pixel(&mut self, x: u16, y: u16, color: u8) {
        self.buffer[(x + (y << 8)) as usize] = color;
    }
}
