pub trait MemAccess {
    fn read(&mut self, address: u16) -> u8;

    fn peek(&self, address: u16) -> u8;

    fn read_u16(&mut self, address: u16) -> u16 {
        let lo = self.read(address);
        let hi = self.read(address.wrapping_add(1));

        u16::from_le_bytes([lo, hi])
    }

    fn peek_u16(&self, address: u16) -> u16 {
        let lo = self.peek(address);
        let hi = self.peek(address.wrapping_add(1));

        u16::from_le_bytes([lo, hi])
    }

    fn write(&mut self, address: u16, value: u8);
}

pub trait Reset {
    fn reset(&mut self);
}
