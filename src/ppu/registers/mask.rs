use crate::util::BitFlag;

pub enum Color {
    Red,
    Green,
    Blue,
}

pub struct Mask;
// 7  bit  0
// ---- ----
// BGRs bMmG
// |||| ||||
// |||| |||+- Greyscale (0: normal color, 1: greyscale)
// |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
// |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
// |||| +---- 1: Enable background rendering
// |||+------ 1: Enable sprite rendering
// ||+------- Emphasize red (green on PAL/Dendy)
// |+-------- Emphasize green (red on PAL/Dendy)
// +--------- Emphasize blue
#[rustfmt::skip]
impl Mask {
    // bit location, not value
    pub const GREYSCALE: u8        = 0;
    pub const SHOW_LEFT_BG: u8     = 1;
    pub const SHOW_LEFT_SPRITE: u8 = 2;
    pub const SHOW_BG: u8          = 3;
    pub const SHOW_SPRITE: u8      = 4;
    pub const EMPHASIZE_RED: u8    = 5;
    pub const EMPHASIZE_GREEN: u8  = 6;
    pub const EMPHASIZE_BLUE: u8   = 7;
}

#[derive(Debug, Default)]
pub struct MaskRegister(u8);

impl MaskRegister {
    pub fn set_byte(&mut self, value: u8) {
        self.0 = value;
    }

    pub fn is_greyscale(&self) -> bool {
        self.0.contains(Mask::GREYSCALE)
    }

    pub fn show_left_bg(&self) -> bool {
        self.0.contains(Mask::SHOW_LEFT_BG)
    }

    pub fn show_left_sprite(&self) -> bool {
        self.0.contains(Mask::SHOW_LEFT_SPRITE)
    }

    pub fn show_background(&self) -> bool {
        self.0.contains(Mask::SHOW_BG)
    }

    pub fn show_sprite(&self) -> bool {
        self.0.contains(Mask::SHOW_SPRITE)
    }

    pub fn emphasize(&self) -> Vec<Color> {
        let mut colors = Vec::new();
        if self.0.contains(Mask::EMPHASIZE_RED) {
            colors.push(Color::Red);
        }
        if self.0.contains(Mask::EMPHASIZE_BLUE) {
            colors.push(Color::Blue);
        }
        if self.0.contains(Mask::EMPHASIZE_GREEN) {
            colors.push(Color::Green);
        }
        colors
    }

    pub fn rendering_enabled(&self) -> bool {
        self.show_background() || self.show_sprite()
    }
}
