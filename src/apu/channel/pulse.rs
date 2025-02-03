use serde::{Deserialize, Serialize};

use super::{
    ApuChannel, EnvelopeGenerator, FrameTrigger, LengthCounter, Sequencer, Timer, TimerTrait,
};

const DUTY_CYCLES: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PulseChannel {
    One,
    Two,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pulse {
    period: usize,
    length_counter: LengthCounter,
    envelope: EnvelopeGenerator,
    sequencer: Sequencer,
    timer: Timer,
    sweep: Sweeper,
}

impl Pulse {
    pub fn new(channel: PulseChannel) -> Self {
        Self {
            period: 0,
            length_counter: LengthCounter::default(),
            envelope: EnvelopeGenerator::default(),
            sequencer: Sequencer::new(&DUTY_CYCLES[0]),
            timer: Timer::new(0, 2),
            sweep: Sweeper::new(channel),
        }
    }

    pub fn write_ctrl(&mut self, value: u8) {
        self.envelope.set_parameters(value);
        self.length_counter.set_halt(value & 0x20 != 0);
        self.update_duty_cycle((value >> 6) as usize);
    }

    pub fn write_sweep(&mut self, value: u8) {
        self.sweep.set_parameters(value);
    }

    pub fn write_timer_lo(&mut self, value: u8) {
        self.set_period((self.period & 0x0700) | value as usize);
    }

    pub fn write_timer_hi(&mut self, value: u8) {
        self.set_period((self.period & 0xff) | (value as usize & 0b111) << 8);
        self.length_counter.load(value >> 3);

        self.envelope.restart();
        self.sequencer.reset();
    }

    fn set_period(&mut self, period: usize) {
        self.period = period;
        self.timer.set_period(period);
        self.sweep.update_target_period(period);
    }

    fn update_duty_cycle(&mut self, duty: usize) {
        self.sequencer.set_sequence(&DUTY_CYCLES[duty]);
    }
}

impl ApuChannel for Pulse {
    fn enabled(&self) -> bool {
        self.length_counter.enabled()
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.length_counter.set_enabled(enabled);
    }

    fn get_output(&self) -> f32 {
        if self.sweep.muted() || self.sequencer.value() == 0 || self.length_counter.count() == 0 {
            0.0
        } else {
            self.envelope.volume()
        }
    }
}

impl FrameTrigger for Pulse {
    fn clock_quarter_frame(&mut self) {
        self.envelope.clock();
    }

    fn clock_half_frame(&mut self) {
        self.clock_quarter_frame();

        self.length_counter.clock();
        if let Some(period) = self.sweep.clock() {
            self.set_period(period as usize);
        }
    }
}

impl TimerTrait for Pulse {
    fn clock(&mut self) {
        if self.timer.clock() {
            self.sequencer.clock();
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Sweeper {
    count: usize,
    enabled: bool,
    current_period: usize,
    target_period: usize,
    negate: bool,
    shift_count: u8,
    is_channel1: bool,
    reload: bool,
}

impl Sweeper {
    fn new(channel: PulseChannel) -> Self {
        Self {
            is_channel1: channel == PulseChannel::One,
            ..Default::default()
        }
    }

    fn set_parameters(&mut self, data: u8) {
        self.shift_count = data & 0b111;
        self.negate = data & 0x8 != 0;
        self.update_target_period((data as usize >> 4) & 0b111);
        self.enabled = data & 0x80 != 0;
    }

    /// Returns new period if the timer should be updated
    fn clock(&mut self) -> Option<u16> {
        self.count = self.count.saturating_sub(1);

        let mut period = None;

        if self.count == 0 && self.enabled && self.shift_count != 0 && !self.muted() {
            period = Some(self.target_period as u16);
        }

        if self.count == 0 || self.reload {
            self.count = self.current_period;
            self.reload = false;
            period = None;
        }

        period
    }

    fn update_target_period(&mut self, pulse_period: usize) {
        let delta = pulse_period >> self.shift_count;

        self.target_period = if self.negate {
            pulse_period
                .saturating_sub(delta)
                .saturating_sub(self.is_channel1 as usize)
        } else {
            pulse_period.saturating_add(delta)
        };
        self.current_period = pulse_period;
    }

    fn muted(&self) -> bool {
        self.current_period < 8 || (!self.negate && self.target_period > 0x7ff)
    }
}
