#[derive(Debug, Default)]
pub struct AddressRegister(u16);

impl AddressRegister {
    pub fn set_high(&mut self, value: u8) {
        self.0 = (self.0 & 0x00ff) | ((u16::from(value) & 0x3f) << 8);
    }

    pub fn set_low(&mut self, value: u8) {
        self.0 = (self.0 & 0xff00) | value as u16;
    }

    pub fn set_nametable_select(&mut self, value: u8) {
        self.0 = (self.0 & 0xf3ff) | ((u16::from(value) & 0x03) << 10);
    }
}
