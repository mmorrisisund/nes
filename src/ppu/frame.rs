use super::TOTAL_PIXELS;

#[derive(Debug)]
pub struct Frame {
    pub buffer: Vec<u8>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            buffer: vec![0; TOTAL_PIXELS * 3],
        }
    }
    pub fn set_pixel(&mut self, x: u16, y: u16, color: u8) {
        let color = 3 * color as usize;
        let idx = 3 * (x + y * super::WIDTH as u16) as usize;

        self.buffer[idx] = super::SYSTEM_PALETTE[color];
        self.buffer[idx + 1] = super::SYSTEM_PALETTE[color + 1];
        self.buffer[idx + 2] = super::SYSTEM_PALETTE[color + 2];
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}
