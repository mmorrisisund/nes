// TODO: Remove this once skeleton of functionality in place
#![allow(dead_code)]

use super::{
    AudioChannel, Divider, EnvelopeGenerator, EnvelopedChannel, LengthCountedChannel,
    LengthCounter, Sequencer, TimedChannel, Timer,
};

const DUTY_CYCLES: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

#[derive(Debug, Default)]
pub enum Channel {
    #[default]
    One,
    Two,
}

#[derive(Debug, Default)]
pub struct SquarePulse {
    channel: Channel,

    envelope: EnvelopeGenerator,
    sweeper: Sweeper,
    timer: Timer,
    sequencer: Sequencer,
    length_counter: LengthCounter,
}

impl SquarePulse {
    pub fn new(channel: Channel) -> Self {
        Self {
            channel,
            sequencer: Sequencer::new(&DUTY_CYCLES[0]),
            ..Default::default()
        }
    }

    pub fn update_duty_cycle(&mut self, duty: u8) {
        self.sequencer.set_sequence(&DUTY_CYCLES[duty as usize]);
    }

    pub fn set_period(&mut self, period: u16) {
        self.timer.set_period(period);
        self.sweeper.update_target_period(period);
    }

    pub fn get_period(&self) -> u16 {
        self.timer.period()
    }

    pub fn sweeper_mut(&mut self) -> &mut Sweeper {
        &mut self.sweeper
    }

    pub fn sequencer_mut(&mut self) -> &mut Sequencer {
        &mut self.sequencer
    }
}

impl AudioChannel for SquarePulse {
    fn get_output(&self) -> f32 {
        if self.sweeper.muted() || self.sequencer.value() == 0 || self.length_counter.count() == 0 {
            self.envelope.volume()
        } else {
            0.0
        }
    }

    fn enabled(&self) -> bool {
        self.length_counter.enabled()
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.length_counter.set_enabled(enabled);
    }
}

impl LengthCountedChannel for SquarePulse {
    fn counter_mut(&mut self) -> &mut LengthCounter {
        &mut self.length_counter
    }
}

impl TimedChannel for SquarePulse {
    fn timer_mut(&mut self) -> &mut Timer {
        &mut self.timer
    }
}

impl EnvelopedChannel for SquarePulse {
    fn envelop_generator_mut(&mut self) -> &mut EnvelopeGenerator {
        &mut self.envelope
    }
}

#[derive(Debug, Default)]
pub struct Sweeper {
    divider: Divider,
    enabled: bool,
    current_period: u16,
    target_period: u16,
    negate: bool,
    shift_count: u8,
    is_channel1: bool,
    reload: bool,
}

impl Sweeper {
    pub fn set_parameters(&mut self, data: u8) {
        self.shift_count = data & 0b111;
        self.negate = data & 0x8 != 0;
        self.update_target_period((data as u16 >> 4) & 0b111);
        self.enabled = data & 0x80 != 0;
    }

    /// Returns new period if the timer should be updated
    pub fn clock(&mut self) -> Option<u16> {
        // this may cause timing issues.  not sure if update and decrement when reset (period reached)
        let divider_clock = self.divider.clock();

        if divider_clock && self.enabled && self.shift_count != 0 {
            if self.muted() {
                None
            } else {
                // see above. this path may not update the divider count
                Some(self.target_period)
            }
        } else if self.reload {
            self.divider.reset();
            self.reload = false;
            None
        } else {
            None
        }
    }

    fn update_target_period(&mut self, pulse_period: u16) {
        let delta = pulse_period >> self.shift_count;

        self.target_period = if self.negate {
            pulse_period
                .saturating_sub(delta)
                .saturating_add(self.is_channel1 as u16)
        } else {
            pulse_period.saturating_add(delta)
        };
        self.current_period = pulse_period;
    }

    fn muted(&self) -> bool {
        self.current_period < 8 || (!self.negate && self.target_period > 0x7ff)
    }
}
