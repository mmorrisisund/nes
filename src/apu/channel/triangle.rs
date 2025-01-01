use super::{ApuChannel, FrameTrigger, LengthCounter, Sequencer, Timer, TimerTrait};

#[rustfmt::skip]
const SEQUENCE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
];

#[derive(Debug)]
pub struct Triangle {
    timer: Timer,
    linear_counter: LinearCounter,
    length_counter: LengthCounter,
    sequencer: Sequencer,
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            timer: Timer::new(0, 1),
            linear_counter: LinearCounter::default(),
            length_counter: LengthCounter::default(),
            sequencer: Sequencer::new(&SEQUENCE),
        }
    }

    pub fn write_ctrl(&mut self, value: u8) {
        let halt = value & 0x80 != 0;

        self.length_counter.set_halt(halt);
        self.linear_counter.set_control(halt);
        self.linear_counter.set_load(value & 0x7f);
    }

    pub fn write_timer_lo(&mut self, value: u8) {
        let period_hi = self.timer.period() & 0x0700;
        self.timer.set_period(period_hi | value as usize);
    }

    pub fn write_timer_hi(&mut self, value: u8) {
        let period_lo = self.timer.period() & 0xff;
        self.timer
            .set_period(period_lo | (value as usize & 0b111) << 8);

        self.length_counter.load(value >> 3);

        self.linear_counter.reload();
    }
}

impl ApuChannel for Triangle {
    fn enabled(&self) -> bool {
        self.length_counter.enabled()
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.length_counter.set_enabled(enabled);
    }

    fn get_output(&self) -> f32 {
        if self.timer.period() < 2 {
            7.5
        } else {
            self.sequencer.value() as f32
        }
    }
}

impl FrameTrigger for Triangle {
    fn clock_quarter_frame(&mut self) {
        self.linear_counter.clock();
    }

    fn clock_half_frame(&mut self) {
        self.length_counter.clock();
    }
}

impl TimerTrait for Triangle {
    fn clock(&mut self) {
        if self.linear_counter.count() != 0 && self.length_counter.count() != 0 {
            self.sequencer.clock();
        }
    }
}

#[derive(Debug, Default)]
pub struct LinearCounter {
    counter: u8,
    period: u8,
    control: bool,
    reload: bool,
}

impl LinearCounter {
    fn clock(&mut self) {
        if self.reload {
            self.counter = self.period
        } else if self.counter > 0 {
            self.counter -= 1;
        }

        if !self.control {
            self.reload = false
        }
    }

    fn count(&self) -> u8 {
        self.counter
    }

    fn set_control(&mut self, enabled: bool) {
        self.control = enabled;
    }

    fn set_load(&mut self, value: u8) {
        self.period = value;
    }

    fn reload(&mut self) {
        self.reload = true;
    }
}
