mod channel;

use std::{cell::Cell, cmp::Ordering};

use channel::{LengthCountedChannel, NoiseChannel, SquarePulseChannel, TriangleChannel};

use crate::common::{bus::Bus, interconnect::InterruptSignaler};

#[derive(Debug, Default)]
pub struct APU {
    pulse1: LengthCountedChannel<SquarePulseChannel>,
    pulse2: LengthCountedChannel<SquarePulseChannel>,
    triangle: LengthCountedChannel<TriangleChannel>,
    noise: LengthCountedChannel<NoiseChannel>,

    frame_cycle: u16,
    frame_cycle_reset_delay: i8,

    frame_interrupt_flag: Cell<bool>, // Status read clears interrupt.  So interior mutability needed
    frame_interrupt_flag_changed: Cell<bool>, // Status read clears interrupt.  So interior mutability needed

    interrupt_disabled: bool,
    sequencer_mode: FrameCounterSequencerMode,
    pending_sequencer_mode: FrameCounterSequencerMode,
}

impl APU {
    pub fn new() -> Self {
        Self {
            frame_cycle: 5, // TODO: cpu has 7 cycle delay at start, must be odd!!??? or jitter test fails, if 0 both 4017 tests fail, if 1 4017 timing test fails
            frame_cycle_reset_delay: -1,
            ..Default::default()
        }
    }

    pub fn cycle_tick(&mut self) {
        self.frame_cycle += 1;

        match self.frame_cycle_reset_delay.cmp(&0) {
            Ordering::Less => {}
            Ordering::Equal => {
                self.frame_cycle_reset_delay -= 1;

                self.frame_cycle = 0;
                self.sequencer_mode = self.pending_sequencer_mode;

                if let FrameCounterSequencerMode::FiveStep = self.sequencer_mode {
                    self.generate_quarter_frame_clock();
                    self.generate_half_frame_clock();
                }
            }
            Ordering::Greater => self.frame_cycle_reset_delay -= 1,
        }

        match self.frame_cycle {
            14913 => {
                self.generate_half_frame_clock();
            }
            29828 => {
                if self.sequencer_mode == FrameCounterSequencerMode::FourStep {
                    self.update_interrupt();
                }
            }
            29829 => {
                if self.sequencer_mode == FrameCounterSequencerMode::FourStep {
                    self.generate_half_frame_clock();

                    self.update_interrupt();
                }
            }
            29830 if self.sequencer_mode == FrameCounterSequencerMode::FourStep => {
                self.update_interrupt();

                self.frame_cycle = 0;
            }
            37281 => {
                self.generate_half_frame_clock();
            }
            37282 => {
                self.frame_cycle = 0;
            }
            _ => {} //ignore
        }
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
                self.pulse1.length_counter_mut().set_halt(data & 0x20 != 0);
            }
            Register::Pulse1_4 => {
                self.pulse1
                    .length_counter_mut()
                    .load_length_counter(data >> 3);
            }

            Register::Pulse2_1 => {
                self.pulse2.length_counter_mut().set_halt(data & 0x20 != 0);
            }
            Register::Pulse2_4 => {
                self.pulse2
                    .length_counter_mut()
                    .load_length_counter(data >> 3);
            }

            Register::Triangle1 => {
                self.triangle
                    .length_counter_mut()
                    .set_halt(data & 0x80 != 0);
            }
            Register::Triangle4 => {
                self.triangle
                    .length_counter_mut()
                    .load_length_counter(data >> 3);
            }

            Register::Noise1 => {
                self.noise.length_counter_mut().set_halt(data & 0x20 != 0);
            }
            Register::Noise4 => {
                self.noise
                    .length_counter_mut()
                    .load_length_counter(data >> 3);
            }

            Register::Status => {
                let pulse1_enabled = (data & 0x1) != 0;
                let pulse2_enabled = (data >> 1 & 0x1) != 0;
                let triangle_enabled = (data >> 2 & 0x1) != 0;
                let noise_enabled = (data >> 3 & 0x1) != 0;

                self.pulse1.length_counter_mut().set_enabled(pulse1_enabled);
                self.pulse2.length_counter_mut().set_enabled(pulse2_enabled);
                self.triangle
                    .length_counter_mut()
                    .set_enabled(triangle_enabled);
                self.noise.length_counter_mut().set_enabled(noise_enabled);
                // TODO:  DMC channels
                // TODO: clear DMC interrupt
            }
            Register::FrameCounter => {
                self.pending_sequencer_mode = data.into();
                self.interrupt_disabled = data & (1 << 6) != 0;

                if self.interrupt_disabled {
                    self.frame_interrupt_flag.set(false);
                    self.frame_interrupt_flag_changed.set(true);
                }
                self.frame_cycle_reset_delay = if self.frame_cycle % 2 == 0 { 3 } else { 2 };
            }
            _ => {}
        }
    }

    fn update_interrupt(&mut self) {
        if !self.interrupt_disabled {
            self.frame_interrupt_flag.set(true);
            self.frame_interrupt_flag_changed.set(true);
        }
    }

    fn generate_quarter_frame_clock(&mut self) {}

    fn generate_half_frame_clock(&mut self) {
        self.pulse1.length_counter_mut().decrement();
        self.pulse2.length_counter_mut().decrement();
        self.triangle.length_counter_mut().decrement();
        self.noise.length_counter_mut().decrement();
    }
}

impl Bus for APU {
    fn reset(&mut self) {
        self.frame_cycle = 7;

        self.write(0x4015, 0x00);

        self.frame_interrupt_flag.set(false);
        self.frame_interrupt_flag_changed.set(true);
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

#[derive(Debug, Default, PartialEq, Clone, Copy)]
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

    mod reset {
        use super::*;

        #[test]
        fn reg_4015_cleared() {
            execute_blargg_test("./src/test_roms/apu/reset/4015_cleared.nes");
        }

        #[test]
        fn reg_4017_timing() {
            execute_blargg_test("./src/test_roms/apu/reset/4017_timing.nes");
        }

        #[test]
        fn reg_4017_written() {
            execute_blargg_test("./src/test_roms/apu/reset/4017_written.nes");
        }

        #[test]
        fn irq_flag_cleared() {
            execute_blargg_test("./src/test_roms/apu/reset/irq_flag_cleared.nes");
        }

        #[test]
        fn len_ctrs_enabled() {
            execute_blargg_test("./src/test_roms/apu/reset/len_ctrs_enabled.nes");
        }

        #[ignore = "channels/dmc need to be fully implemented"]
        #[test]
        fn works_immediately() {
            execute_blargg_test("./src/test_roms/apu/reset/works_immediately.nes");
        }
    }

    mod basics {
        use super::*;

        #[test]
        fn len_ctrs() {
            execute_blargg_test("./src/test_roms/apu/basics/rom_singles/1-len_ctr.nes");
        }

        #[test]
        fn len_table() {
            execute_blargg_test("./src/test_roms/apu/basics/rom_singles/2-len_table.nes");
        }

        #[test]
        fn irq_flag() {
            execute_blargg_test("./src/test_roms/apu/basics/rom_singles/3-irq_flag.nes");
        }

        #[test]
        fn jitter() {
            execute_blargg_test("./src/test_roms/apu/basics/rom_singles/4-jitter.nes");
        }

        #[test]
        fn len_timing() {
            execute_blargg_test("./src/test_roms/apu/basics/rom_singles/5-len_timing.nes");
        }

        #[test]
        fn irq_flag_timing() {
            execute_blargg_test("./src/test_roms/apu/basics/rom_singles/6-irq_flag_timing.nes");
        }
    }
}
