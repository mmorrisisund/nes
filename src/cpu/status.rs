use std::fmt::UpperHex;

use serde::{Deserialize, Serialize};

use crate::util::BitFlag;

#[derive(Default)]
pub struct Status;

#[rustfmt::skip]
impl Status {
    // These are the bit locations, not values
    pub const CARRY: u8       = 0;
    pub const ZERO: u8        = 1;
    pub const INT_DISABLE: u8 = 2;
    pub const DECIMAL: u8     = 3;
    pub const BREAK: u8       = 4;
    const UNUSED: u8          = 5;
    pub const OVERFLOW: u8    = 6;
    pub const NEGATIVE: u8    = 7;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StatusRegister(u8);

impl From<u8> for StatusRegister {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl Default for StatusRegister {
    fn default() -> Self {
        Self(Status::INT_DISABLE | Status::UNUSED)
    }
}

impl UpperHex for StatusRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UpperHex::fmt(&self.0, f)
    }
}

impl StatusRegister {
    pub fn value(&self) -> u8 {
        self.0
    }

    pub fn set_flag(&mut self, flag: u8) {
        self.0.set_flag(flag);
    }

    pub fn unset_flag(&mut self, flag: u8) {
        self.0.unset_flag(flag);
    }

    pub fn set(&mut self, flag: u8, is_set: bool) {
        if is_set {
            self.set_flag(flag);
        } else {
            self.unset_flag(flag);
        }
    }

    pub fn is_set(&self, flag: u8) -> bool {
        self.0.contains(flag)
    }

    pub fn set_zero(&mut self, value: u8) {
        self.set(Status::ZERO, value == 0);
    }

    pub fn set_negative(&mut self, value: u8) {
        self.set(Status::NEGATIVE, value & 0x80 != 0);
    }

    pub fn reset(&mut self) {
        self.0 = Status::INT_DISABLE | Status::UNUSED;
    }
}
