use serde::{Deserialize, Serialize};

use crate::util::BitFlag;

pub struct Status;
// VSO. ....
// |||+-++++- PPU open bus. Returns stale PPU bus contents.
// ||+------- Sprite overflow. The intent was for this flag to be set
// ||         whenever more than eight sprites appear on a scanline, but a
// ||         hardware bug causes the actual behavior to be more complicated
// ||         and generate false positives as well as false negatives; see
// ||         PPU sprite evaluation. This flag is set during sprite
// ||         evaluation and cleared at dot 1 (the second dot) of the
// ||         pre-render line.
// |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
// |          a nonzero background pixel; cleared at dot 1 of the pre-render
// |          line.  Used for raster timing.
// +--------- Vertical blank has started (0: not in vblank; 1: in vblank)
//            Set at dot 1 of line 241 (the line *after* the post-render
//            line); cleared after reading $2002 and at dot 1 of the
//            pre-render line.
#[rustfmt::skip]
impl Status {
    // bit location, not value
    const SPRITE_OVERFLOW: u8 = 5;
    const SPRITE_ZERO_HIT: u8 = 6;
    const VBLANK_STARTED: u8  = 7;
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StatusRegister(u8);

impl StatusRegister {
    pub fn value(&self) -> u8 {
        self.0
    }

    pub fn set_sprite_overflow(&mut self) {
        self.0.set_flag(Status::SPRITE_OVERFLOW);
    }

    pub fn set_sprite_zero_hit(&mut self) {
        self.0.set_flag(Status::SPRITE_ZERO_HIT);
    }

    pub fn set_vblank_started(&mut self) {
        self.0.set_flag(Status::VBLANK_STARTED);
    }

    pub fn clear_vblank_started(&mut self) {
        self.0.unset_flag(Status::VBLANK_STARTED);
    }

    pub fn is_in_vblank(&self) -> bool {
        self.0.contains(Status::VBLANK_STARTED)
    }

    pub fn clear(&mut self) {
        self.0 = 0b0001_1111; // TODO: can this just set to 0 since other bits are open bus
    }
}
