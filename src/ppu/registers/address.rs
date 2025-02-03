use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct AddressRegister(u16);

// The 15 bit registers t and v are composed this way during rendering:
// yyy NN YYYYY XXXXX
// ||| || ||||| +++++-- coarse X scroll
// ||| || +++++-------- coarse Y scroll
// ||| |+-------------- nametable x offset
// ||| +--------------- nametable y offset
// +++----------------- fine Y scroll
pub const COARSE_X_MASK: u16 = 0x1f;
pub const COARSE_Y_MASK: u16 = 0x3e0;
pub const NT_SELECT_MASK: u16 = 0xc00;
pub const NT_X_MASK: u16 = 0x400;
pub const NT_Y_MASK: u16 = 0x800;
pub const FINE_Y_MASK: u16 = 0x7000;
#[allow(clippy::unusual_byte_groupings)]
const H_BITS_MASK: u16 = 0b000_01_00000_11111;
#[allow(clippy::unusual_byte_groupings)]
const V_BITS_MASK: u16 = 0b111_10_11111_00000;

pub const NT_START: u16 = 0x2000;
pub const NT_ADDR_MASK: u16 = 0xfff;
pub const ATTR_START: u16 = 0x23c0;

impl AddressRegister {
    pub fn address(&self) -> u16 {
        self.0 & 0x3fff
    }

    pub fn copy_x(&mut self, other: AddressRegister) {
        self.0 = (self.0 & !H_BITS_MASK) | other.0 & H_BITS_MASK
    }

    pub fn copy_y(&mut self, other: AddressRegister) {
        self.0 = (self.0 & !V_BITS_MASK) | other.0 & V_BITS_MASK
    }

    pub fn increment_addr(&mut self, offset: u8) {
        self.0 = self.0.wrapping_add(offset as u16);
    }

    //  Bits 0-4 are incremented, with overflow toggling bit 10.
    // This means that bits 0-4 count from 0 to 31 across a single nametable, and bit 10 selects the current nametable horizontally.
    // https://www.nesdev.org/wiki/PPU_scrolling#Coarse_X_increment
    pub fn increment_x(&mut self) {
        let coarse_x = self.coarse_x() as u8;

        if coarse_x == 31 {
            self.set_coarse_x(0);
            self.0 ^= 0x400; // switch x nametable
        } else {
            self.set_coarse_x(coarse_x + 1);
        }
    }

    // If rendering is enabled, fine Y is incremented at dot 256 of each scanline, overflowing to coarse Y,
    // and finally adjusted to wrap among the nametables vertically.
    // Bits 12-14 are fine Y. Bits 5-9 are coarse Y. Bit 11 selects the vertical nametable.
    // https://www.nesdev.org/wiki/PPU_scrolling#Y_increment
    pub fn increment_y(&mut self) {
        let fine_y = self.get_fine_y();

        if fine_y < 7 {
            self.set_fine_y(fine_y + 1);
        } else {
            self.set_fine_y(0);

            let coarse_y = (self.0 & 0x03e0) >> 5; // coarse y
                                                   // Row 29 is the last row of tiles in a nametable.
            if coarse_y == 29 {
                self.set_coarse_y(0);
                self.0 ^= 0x800; // switch y nametable
            } else if coarse_y == 31 {
                self.set_coarse_y(0);
            } else {
                self.set_coarse_y(coarse_y as u8 + 1);
            }
        }
    }

    // The high bits of v are used for fine Y during rendering, and addressing nametable data only requires 12 bits, with the high 2 CHR address lines fixed to the 0x2000 region. The address to be fetched during rendering can be deduced from v in the following way:
    //  tile address      = 0x2000 | (v & 0x0FFF)
    // https://www.nesdev.org/wiki/PPU_scrolling#Tile_and_attribute_fetching
    pub fn nametable_addr(&self) -> u16 {
        NT_START | (self.0 & NT_ADDR_MASK)
    }

    //  attribute address = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)
    // The low 12 bits of the attribute address are composed in the following way:
    //  NN 1111 YYY XXX
    //  || |||| ||| +++-- high 3 bits of coarse X (x/4)
    //  || |||| +++------ high 3 bits of coarse Y (y/4)
    //  || ++++---------- attribute offset (960 bytes) 0x3c0
    //  ++--------------- nametable select
    // https://www.nesdev.org/wiki/PPU_scrolling#Tile_and_attribute_fetching
    pub fn attribute_addr(&self) -> u16 {
        let nt_select = self.0 & (NT_X_MASK | NT_Y_MASK);
        let y_bits = (self.0 >> 4) & 0x38;
        let x_bits = (self.0 >> 2) & 0x07;

        ATTR_START | nt_select | y_bits | x_bits
    }

    // gets palette index for given quadrant of attribute byte
    // attr_value = (bottomright << 6) | (bottomleft << 4) | (topright << 2) | (topleft << 0)
    pub fn attribute_shift(&self) -> u16 {
        (self.coarse_y() & 0x02) << 1 | (self.coarse_x() & 0x02)
    }

    pub fn set_high(&mut self, value: u8) {
        self.0 = (self.0 & 0x00ff) | ((u16::from(value) & 0x3f) << 8);
    }

    pub fn set_low(&mut self, value: u8) {
        self.0 = (self.0 & 0xff00) | value as u16;
    }

    pub fn coarse_x(&self) -> u16 {
        self.0 & COARSE_X_MASK
    }

    pub fn set_coarse_x(&mut self, value: u8) {
        self.0 = (self.0 & !COARSE_X_MASK) | value as u16;
    }

    pub fn coarse_y(&self) -> u16 {
        (self.0 & COARSE_Y_MASK) >> 5 // FIXME: pretty sure this should be a shift of 5, but looks worse
    }

    pub fn set_coarse_y(&mut self, value: u8) {
        self.0 = (self.0 & !COARSE_Y_MASK) | (u16::from(value) << 5);
    }

    pub fn set_nametable_select(&mut self, value: u8) {
        self.0 = (self.0 & !NT_SELECT_MASK) | (u16::from(value) << 10);
    }

    pub fn get_fine_y(&self) -> u8 {
        ((self.0 & FINE_Y_MASK) >> 12) as u8
    }

    pub fn set_fine_y(&mut self, value: u8) {
        self.0 = (self.0 & !FINE_Y_MASK) | (u16::from(value) << 12);
    }
}
