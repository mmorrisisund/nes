use serde::{Deserialize, Serialize};

use super::{ApuChannel, EnvelopeGenerator, FrameTrigger, LengthCounter, Timer, TimerTrait};

const PERIOD_LENGTH: [usize; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

#[derive(Debug, Serialize, Deserialize)]
pub struct Noise {
    timer: Timer,
    shift: ShiftRegister,
    length_counter: LengthCounter,
    envelope: EnvelopeGenerator,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            timer: Timer::default(),
            shift: ShiftRegister::new(),
            length_counter: LengthCounter::default(),
            envelope: EnvelopeGenerator::default(),
        }
    }

    pub fn write_ctrl(&mut self, value: u8) {
        self.length_counter.set_halt((value & 0x20) != 0);
        self.envelope.set_parameters(value);
    }

    pub fn write_timer(&mut self, value: u8) {
        self.timer.set_period(PERIOD_LENGTH[value as usize & 0xf]);

        if value & 0x80 != 0 {
            self.shift.set_mode(ShiftMode::One);
        } else {
            self.shift.set_mode(ShiftMode::Zero);
        }
    }

    pub fn write_length(&mut self, value: u8) {
        self.length_counter.load(value >> 3);
        self.envelope.restart();
    }
}

impl ApuChannel for Noise {
    fn enabled(&self) -> bool {
        self.length_counter.enabled()
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.length_counter.set_enabled(enabled);
    }

    fn get_output(&self) -> f32 {
        if self.shift.active() && self.length_counter.enabled() {
            self.envelope.volume()
        } else {
            0.0
        }
    }
}

impl FrameTrigger for Noise {
    fn clock_quarter_frame(&mut self) {
        self.envelope.clock();
    }

    fn clock_half_frame(&mut self) {
        self.length_counter.clock();
    }
}

impl TimerTrait for Noise {
    fn clock(&mut self) {
        self.shift.clock();
    }
}

enum ShiftMode {
    /// Zero - XOR bits 0 and 1
    Zero,
    /// One - XOR bits 0 and 6
    One,
}

#[derive(Debug, Serialize, Deserialize)]
struct ShiftRegister {
    value: u16,
    mode_shift: usize,
}

impl ShiftRegister {
    fn new() -> Self {
        Self {
            value: 1,
            mode_shift: 1,
        }
    }

    fn clock(&mut self) {
        let bit0 = self.value & 0x1;
        let other = (self.value >> self.mode_shift) & 0x1;
        let feedback = bit0 ^ other;

        self.value >>= 1;

        self.value |= feedback << 14;
    }

    fn set_mode(&mut self, mode: ShiftMode) {
        match mode {
            ShiftMode::Zero => self.mode_shift = 1,
            ShiftMode::One => self.mode_shift = 6,
        }
    }

    fn active(&self) -> bool {
        self.value & 0x1 != 1
    }
}
