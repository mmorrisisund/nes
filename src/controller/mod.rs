use enumflags2::{bitflags, BitFlag, BitFlags};

#[derive(Debug, Default)]
pub struct Joypad {
    state: BitFlags<JoypadButton>,
    index: u8,
    strobe: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            state: JoypadButton::from_bits(0).unwrap(),
            index: 0,
            strobe: false,
        }
    }

    pub fn set_button(&mut self, button: JoypadButton, pressed: bool) {
        self.state.set(button, pressed);
    }

    pub fn write(&mut self, value: u8) {
        self.strobe = value & 0x1 == 0x1;

        if self.strobe {
            self.index = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        let val = self.peek();
        if !self.strobe && self.index < 8 {
            self.index += 1;
        }
        val
    }

    pub fn peek(&self) -> u8 {
        if self.index < 8 {
            let button = JoypadButton::from_bits(1 << self.index).unwrap();
            self.state.contains(button).into()
        } else {
            self.state.contains(JoypadButton::A).into()
        }
    }
}

#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JoypadButton {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}
