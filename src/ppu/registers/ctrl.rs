use crate::util::BitFlag;

const NAMETABLE1: u16 = 0x2000;
const NAMETABLE2: u16 = 0x2400;
const NAMETABLE3: u16 = 0x2800;
const NAMETABLE4: u16 = 0x2c00;

pub struct Control;
// 7  bit  0
// ---- ----
// VPHB SINN
// |||| ||||
// |||| ||++- Base nametable address
// |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
// |||| |+--- VRAM address increment per CPU read/write of PPUDATA
// |||| |     (0: add 1, going across; 1: add 32, going down)
// |||| +---- Sprite pattern table address for 8x8 sprites
// ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
// |||+------ Background pattern table address (0: $0000; 1: $1000)
// ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels – see PPU OAM#Byte 1)
// |+-------- PPU master/slave select
// |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
// +--------- Vblank NMI enable (0: off, 1: on)
#[rustfmt::skip]
impl Control {
    // bit location, not value
    pub const NAME_TABLE1: u8         = 0;
    pub const NAME_TABLE2: u8         = 1;
    pub const VRAM_INCREMENT: u8      = 2;
    pub const SPRITE_PATTERN_ADDR: u8 = 3;
    pub const BG_PATTERN_ADDR: u8     = 4;
    pub const SPRITE_SIZE: u8         = 5;
    pub const MAIN_SUB_SELECT: u8     = 6;
    pub const NMI_ENABLE: u8          = 7;
}

#[derive(Debug, Default)]
pub struct ControlRegister(u8);

impl ControlRegister {
    pub fn write(&mut self, value: u8) {
        self.0 = value;
    }

    pub fn nametable(&self) -> u16 {
        match self.0 {
            0 => NAMETABLE1,
            1 => NAMETABLE2,
            2 => NAMETABLE3,
            3 => NAMETABLE4,
            _ => unreachable!("impossible nametable"),
        }
    }

    pub fn vram_increment(&self) -> u8 {
        match self.0.contains(Control::VRAM_INCREMENT) {
            true => 32,
            false => 1,
        }
    }

    pub fn sprite_pattern_addr(&self) -> u16 {
        match self.0.contains(Control::SPRITE_PATTERN_ADDR) {
            true => 0x1000,
            false => 0x0000,
        }
    }

    pub fn bg_pattern_addr(&self) -> u16 {
        match self.0.contains(Control::BG_PATTERN_ADDR) {
            true => 0x1000,
            false => 0x0000,
        }
    }

    pub fn sprite_size(&self) -> u16 {
        match self.0.contains(Control::SPRITE_SIZE) {
            true => 16,
            false => 8,
        }
    }

    pub fn main_sub_select(&self) -> u8 {
        match self.0.contains(Control::MAIN_SUB_SELECT) {
            true => 1,
            false => 0,
        }
    }

    pub fn nmi_enable(&self) -> bool {
        self.0.contains(Control::NMI_ENABLE)
    }
}
