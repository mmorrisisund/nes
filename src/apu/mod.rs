mod channel;
mod frame_counter;

use serde::{Deserialize, Serialize};

pub use channel::Channel;
use channel::{
    dmc::Dmc,
    noise::Noise,
    pulse::{Pulse, PulseChannel},
    triangle::Triangle,
    ApuChannel, FrameTrigger, TimerTrait,
};
use frame_counter::{FrameCounter, FrameType};

use crate::common::{interconnect::InterruptSignaler, mem::Reset};

const CPU_FREQ: f32 = 1_789_773.0;
const SAMPLE_RATE: f32 = 44_100.0;

pub trait ApuRegisters {
    fn write_ctrl(&mut self, channel: Channel, value: u8);
    fn write_sweep(&mut self, channel: Channel, value: u8);
    fn write_timer_lo(&mut self, channel: Channel, value: u8);
    fn write_timer_hi(&mut self, channel: Channel, value: u8);
    fn write_length(&mut self, channel: Channel, value: u8);
    fn write_dmc_output(&mut self, value: u8);
    fn write_dmc_addr(&mut self, value: u8);

    fn read_status(&mut self) -> u8;
    fn peek_status(&self) -> u8;
    fn write_status(&mut self, value: u8);
    fn write_frame_counter(&mut self, value: u8);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Apu {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,

    sample_period: f32,
    sample_count: f32,
    sample_buffer: Vec<f32>,

    frame_counter: FrameCounter,
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

impl Apu {
    pub fn new() -> Self {
        Self {
            pulse1: Pulse::new(PulseChannel::One),
            pulse2: Pulse::new(PulseChannel::Two),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: Dmc::new(),

            sample_period: CPU_FREQ / SAMPLE_RATE,
            sample_count: 0.,
            sample_buffer: Vec::new(),

            frame_counter: FrameCounter::new(),
        }
    }

    pub fn clock_cycle(&mut self) {
        self.sample_count += 1.;

        if self.sample_count > self.sample_period {
            self.sample_buffer.push(self.mixer_output());
            self.sample_count = 0.;
        }

        self.clock_timers();

        self.frame_counter.clock(|typ| match typ {
            FrameType::Quarter => {
                self.pulse1.clock_quarter_frame();
                self.pulse2.clock_quarter_frame();
                self.triangle.clock_quarter_frame();
                self.noise.clock_quarter_frame();
            }
            FrameType::Half => {
                self.pulse1.clock_half_frame();
                self.pulse2.clock_half_frame();
                self.triangle.clock_half_frame();
                self.noise.clock_half_frame();
            }
        });
    }

    pub fn drain_audio_buffer(&mut self) -> Vec<f32> {
        self.sample_buffer.drain(..).collect()
    }

    pub fn dmc_write(&mut self, value: u8) {
        self.dmc.set_sample_buffer(value);
    }

    fn mixer_output(&self) -> f32 {
        let p1 = self.pulse1.get_output();
        let p2 = self.pulse2.get_output();
        let t = self.triangle.get_output();
        let n = self.noise.get_output();
        let d = self.dmc.get_output();

        let p = if p1 == 0. && p2 == 0. {
            0.
        } else {
            95.88 / ((8128. / (p1 + p2)) + 100.)
        };

        let tnd = if t == 0. && n == 0. && d == 0. {
            0.
        } else {
            159.79 / (1. / ((t / 8227.) + (n / 12241.) + (d / 22638.)) + 100.)
        };
        p + tnd
    }

    fn clock_timers(&mut self) {
        self.pulse1.clock();
        self.pulse2.clock();
        self.triangle.clock();
        self.noise.clock();
        self.dmc.clock();
    }
}

impl ApuRegisters for Apu {
    fn write_ctrl(&mut self, channel: Channel, value: u8) {
        match channel {
            Channel::Pulse1 => self.pulse1.write_ctrl(value),
            Channel::Pulse2 => self.pulse2.write_ctrl(value),
            Channel::Triangle => self.triangle.write_ctrl(value),
            Channel::Noise => self.noise.write_ctrl(value),
            _ => panic!("{channel:?} does not have a control register"),
        }
    }

    fn write_sweep(&mut self, channel: Channel, value: u8) {
        match channel {
            Channel::Pulse1 => self.pulse1.write_sweep(value),
            Channel::Pulse2 => self.pulse2.write_sweep(value),
            _ => panic!("{channel:?} does not have a sweep register"),
        }
    }

    fn write_timer_lo(&mut self, channel: Channel, value: u8) {
        match channel {
            Channel::Pulse1 => self.pulse1.write_timer_lo(value),
            Channel::Pulse2 => self.pulse2.write_timer_lo(value),
            Channel::Triangle => self.triangle.write_timer_lo(value),
            Channel::Noise => self.noise.write_timer(value),
            Channel::Dmc => self.dmc.write_timer(value),
        }
    }

    fn write_timer_hi(&mut self, channel: Channel, value: u8) {
        match channel {
            Channel::Pulse1 => self.pulse1.write_timer_hi(value),
            Channel::Pulse2 => self.pulse2.write_timer_hi(value),
            Channel::Triangle => self.triangle.write_timer_hi(value),
            _ => panic!("{channel:?} does not have a timer hi register"),
        }
    }

    fn write_length(&mut self, channel: Channel, value: u8) {
        match channel {
            Channel::Noise => self.noise.write_length(value),
            Channel::Dmc => self.dmc.set_length(value),
            _ => panic!("{channel:?} does not have a length register"),
        }
    }

    fn read_status(&mut self) -> u8 {
        let status = self.peek_status();
        self.frame_counter.clear_interrupt();
        status
    }

    fn peek_status(&self) -> u8 {
        let pulse1_enabled = self.pulse1.enabled() as u8;
        let pulse2_enabled = self.pulse2.enabled() as u8;
        let triangle_enabled = self.triangle.enabled() as u8;
        let noise_enabled = self.noise.enabled() as u8;
        let dmc_enabled = self.dmc.enabled() as u8;
        let frame_interrupt = self.frame_counter.interrupt_status() as u8;
        let dmc_interrupt = self.dmc.interrupt_status() as u8;

        dmc_interrupt << 7
            | frame_interrupt << 6
            | dmc_enabled << 4
            | noise_enabled << 3
            | triangle_enabled << 2
            | pulse2_enabled << 1
            | pulse1_enabled
    }

    fn write_status(&mut self, value: u8) {
        self.pulse1.set_enabled((value & 0x1) != 0);
        self.pulse2.set_enabled((value >> 1 & 0x1) != 0);
        self.triangle.set_enabled((value >> 2 & 0x1) != 0);
        self.noise.set_enabled((value >> 3 & 0x1) != 0);
        self.dmc.set_enabled((value >> 4 & 0x1) != 0);

        self.dmc.clear_interrupt();
    }

    fn write_frame_counter(&mut self, value: u8) {
        self.frame_counter.set_register_value(value);
    }

    fn write_dmc_output(&mut self, value: u8) {
        self.dmc.set_output(value);
    }

    fn write_dmc_addr(&mut self, value: u8) {
        self.dmc.set_address(value);
    }
}

impl Reset for Apu {
    fn reset(&mut self) {
        self.frame_counter.reset();
        self.write_status(0x00);
    }
}

impl InterruptSignaler for Apu {
    fn interrupt_status(&self) -> bool {
        self.frame_counter.interrupt_status()
    }

    fn dma_status(&self) -> Option<u16> {
        self.dmc.dma_status()
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

        #[test]
        fn dmc_basics() {
            execute_blargg_test("./src/test_roms/apu/basics/rom_singles/7-dmc_basics.nes");
        }

        #[test]
        fn dmc_rates() {
            execute_blargg_test("./src/test_roms/apu/basics/rom_singles/8-dmc_rates.nes");
        }
    }
}
