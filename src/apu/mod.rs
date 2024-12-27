mod channel;

use std::cell::Cell;

use channel::{LengthCountedChannel, SquarePulseChannel};

use crate::common::{bus::Bus, interconnect::InterruptSignaler};

#[derive(Debug, Default)]
pub struct APU {
    pulse1: LengthCountedChannel<SquarePulseChannel>,
    pulse2: LengthCountedChannel<u8>,
    triangle: LengthCountedChannel<u8>,
    noise: LengthCountedChannel<u8>,

    cycle: u16,

    frame_interrupt_flag: Cell<bool>, // Status read clears interrupt.  So interior mutability needed
    frame_interrupt_flag_changed: Cell<bool>, // Status read clears interrupt.  So interior mutability needed

    interrupt_disabled: bool,
    sequencer_mode: FrameCounterSequencerMode,

    reset_delay: u8, // delay after power on/reset
}

impl APU {
    pub fn new() -> Self {
        Self {
            cycle: 7, // cpu has 7 cycle delay at start
            frame_interrupt_flag: Cell::new(true),
            frame_interrupt_flag_changed: Cell::new(false),
            ..Default::default()
        }
    }

    pub fn cycle_tick(&mut self) {
        self.cycle += 1;

        if self.reset_delay > 0 {
            self.reset_delay -= 1;
            return;
        }

        match self.cycle {
            14913 => {
                self.pulse1.length_counter_mut().decrement();
            }
            29828 => {
                if self.sequencer_mode == FrameCounterSequencerMode::FourStep {
                    self.update_interrupt();
                }
            }
            29829 => {
                if self.sequencer_mode == FrameCounterSequencerMode::FourStep {
                    self.pulse1.length_counter_mut().decrement();

                    self.update_interrupt();
                }
            }
            29830 if self.sequencer_mode == FrameCounterSequencerMode::FourStep => {
                self.update_interrupt();

                self.cycle = 0;
            }
            37281 => {
                self.pulse1.length_counter_mut().decrement();

                self.cycle = 0
            }
            _ => {} //ignore
        }
        // clock channels, set irq
    }

    fn read_register(&self, register: Register) -> u8 {
        match register {
            Register::Status => {
                // Set status for NT21, DMC Channels
                let pulse1_enabled = (self.pulse1.length_counter().counter() != 0) as u8;
                let pulse2_enabled = (self.pulse2.length_counter().counter() != 0) as u8;
                let triangle_enabled = (self.triangle.length_counter().counter() != 0) as u8;
                let noise_enabled = (self.noise.length_counter().counter() != 0) as u8;

                let frame_interrupt = self.frame_interrupt_flag.get() as u8;

                self.frame_interrupt_flag.set(false);
                self.frame_interrupt_flag_changed.set(true);

                frame_interrupt << 6
                    | noise_enabled << 3
                    | triangle_enabled << 2
                    | pulse2_enabled << 1
                    | pulse1_enabled
            }
            _ => 0, // REVIEW: invalid for all other reads.  should this panic?
        }
    }

    fn write_register(&mut self, register: Register, data: u8) {
        match register {
            Register::Pulse1_1 => {
                let duty = data >> 6;
                let halt = data >> 5;
                let constant_volume = data >> 4;
                let volume = data & 0x0f;

                self.pulse1.channel_mut().duty = duty;
                self.pulse1.channel_mut().halt = halt & 0x01 != 0;
                self.pulse1.channel_mut().constant_volume = constant_volume & 0x01 != 0;
                self.pulse1.channel_mut().volume = volume;
            }
            Register::Pulse1_2 => {}
            Register::Pulse1_3 => {}
            Register::Pulse1_4 => {
                self.pulse1
                    .length_counter_mut()
                    .load_length_counter(data >> 3);
            }
            Register::Pulse2_4 => {
                self.pulse2
                    .length_counter_mut()
                    .load_length_counter(data >> 3);
            }
            Register::Triangle4 => {
                self.triangle
                    .length_counter_mut()
                    .load_length_counter(data >> 3);
            }
            Register::Noise4 => {
                self.noise
                    .length_counter_mut()
                    .load_length_counter(data >> 3);
            }
            Register::Status => {
                let pulse1_enabled = data & 0x1 != 0;

                self.pulse1.length_counter_mut().set_enabled(pulse1_enabled);
                // TODO: enable/disable NT21, DMC channels
                // TODO: clear DMC interrupt
            }
            Register::FrameCounter => {
                self.sequencer_mode = data.into();
                self.interrupt_disabled = data & (1 << 6) != 0;

                if self.interrupt_disabled {
                    self.frame_interrupt_flag.set(false);
                    self.frame_interrupt_flag_changed.set(true);
                }

                // Temporary
                // self.channel1_1 = data & 0x01;
            }
            _ => {}
        }
    }

    fn get_frame_counter_reg(&self) -> u8 {
        let mut result = 0;

        result |= if let FrameCounterSequencerMode::FiveStep = self.sequencer_mode {
            1 << 7
        } else {
            0x00
        };

        result |= if self.interrupt_disabled {
            1 << 6
        } else {
            0x00
        };

        result
    }

    fn update_interrupt(&mut self) {
        if !self.interrupt_disabled {
            self.frame_interrupt_flag.set(true);
            self.frame_interrupt_flag_changed.set(true);
        }
    }
}

impl Bus for APU {
    fn reset(&mut self) {
        let prior = self.get_frame_counter_reg();
        self.cycle = 7;
        self.write(0x4015, 0x00);
        self.write(0x4017, prior);
    }

    fn read(&self, address: u16) -> u8 {
        let register = address.try_into();
        match register {
            Ok(register) => self.read_register(register),
            _ => panic!("Invalid APU register read: {address:04X}"),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        let register = address.try_into();

        match register {
            Ok(register) => self.write_register(register, data),
            _ => panic!("Invalid APU register write: {address:04X}"),
        }
    }
}

impl InterruptSignaler for APU {
    fn interrupt_status(&self) -> bool {
        self.frame_interrupt_flag.get()
    }

    fn interrupt_status_changed(&self) -> bool {
        self.frame_interrupt_flag_changed.get()
    }

    fn clear_interrupt(&mut self) {
        self.frame_interrupt_flag_changed.set(false);
        // TODO: clear dmc flag here too
    }
}

mem_mapped_registers! {
enum Register {
    Pulse1_1 = 0x4000,
    Pulse1_2 = 0x4001,
    Pulse1_3 = 0x4002,
    Pulse1_4 = 0x4003,

    Pulse2_1 = 0x4004,
    Pulse2_2 = 0x4005,
    Pulse2_3 = 0x4006,
    Pulse2_4 = 0x4007,

    Triangle1 = 0x4008,
    Triangle2 = 0x4009, // unused
    Triangle3 = 0x400a,
    Triangle4 = 0x400b,

    Noise1 = 0x400c,
    Noise2 = 0x400d,    // unused
    Noise3 = 0x400e,
    Noise4 = 0x400f,

    DMC1 = 0x4010,
    DMC2 = 0x4011,
    DMC3 = 0x4012,
    DMC4 = 0x4013,

    Status = 0x4015,
    FrameCounter = 0x4017,
}
}

#[derive(Debug, Default, PartialEq)]
enum FrameCounterSequencerMode {
    #[default]
    FourStep,
    FiveStep,
}

impl From<u8> for FrameCounterSequencerMode {
    fn from(value: u8) -> Self {
        if value & 0x80 == 0 {
            Self::FourStep
        } else {
            Self::FiveStep
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::test_helpers::execute_blargg_test;

    #[test]
    fn reset() {
        execute_blargg_test("./src/test_roms/apu/reset/4015_cleared.nes");
        execute_blargg_test("./src/test_roms/apu/reset/4017_written.nes");
    }
}
