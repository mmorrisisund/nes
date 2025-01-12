#[derive(Debug, Default, Clone, Copy)]
pub struct AddressRegister(u16);

// The 15 bit registers t and v are composed this way during rendering:
// yyy NN YYYYY XXXXX
// ||| || ||||| +++++-- coarse X scroll
// ||| || +++++-------- coarse Y scroll
// ||| ++-------------- nametable select
// +++----------------- fine Y scroll
const COARSE_X_MASK: u16 = 0x1f;
const COARSE_Y_MASK: u16 = 0x3e0;
const NT_SELECT_MASK: u16 = 0xc00;
const FINE_Y_MASK: u16 = 0x7000;

impl AddressRegister {
    pub fn get_addr(&self) -> u16 {
        self.0 & 0x3fff
    }

    pub fn increment_addr(&mut self, offset: u8) {
        self.0 = self.0.wrapping_add(offset as u16);
    }

    pub fn set_high(&mut self, value: u8) {
        self.0 = (self.0 & 0x00ff) | ((u16::from(value) & 0x3f) << 8);
    }

    pub fn set_low(&mut self, value: u8) {
        self.0 = (self.0 & 0xff00) | value as u16;
    }

    pub fn set_coarse_x(&mut self, value: u8) {
        self.0 = (self.0 & !COARSE_X_MASK) | value as u16;
    }

    pub fn set_coarse_y(&mut self, value: u8) {
        self.0 = (self.0 & !COARSE_Y_MASK) | (u16::from(value) << 5);
    }

    pub fn set_nametable_select(&mut self, value: u8) {
        self.0 = (self.0 & !NT_SELECT_MASK) | (u16::from(value) << 10);
    }

    pub fn set_fine_y(&mut self, value: u8) {
        self.0 = (self.0 & !FINE_Y_MASK) | (u16::from(value) << 12);
    }
}
