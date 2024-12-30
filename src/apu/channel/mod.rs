pub mod square;

pub trait AudioChannel {
    fn enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);

    fn get_output(&self) -> f32;
}

pub trait LengthCountedChannel {
    fn counter_mut(&mut self) -> &mut LengthCounter;
}

pub trait TimedChannel {
    fn timer_mut(&mut self) -> &mut Timer;
}

pub trait EnvelopedChannel {
    fn envelop_generator_mut(&mut self) -> &mut EnvelopeGenerator;
}

const LENGTH_TABLE: [u8; 0x20] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

#[derive(Debug, Default)]
pub struct LengthCounter {
    counter: u8,
    enabled: bool,
    halt: bool,
}

impl LengthCounter {
    pub fn load(&mut self, index: u8) {
        debug_assert!(index <= 0x1f);

        if self.enabled {
            self.counter = LENGTH_TABLE[index as usize];
        }
    }

    pub fn count(&self) -> u8 {
        self.counter
    }

    pub fn enabled(&self) -> bool {
        self.counter != 0
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        if !self.enabled {
            self.counter = 0;
        }
    }

    pub fn set_halt(&mut self, halt: bool) {
        self.halt = halt
    }

    pub fn clock(&mut self) {
        self.counter = if self.enabled {
            if !self.halt {
                self.counter.saturating_sub(1)
            } else {
                self.counter
            }
        } else {
            0
        };
    }
}

#[derive(Debug, Default)]
pub struct LengthCountedChannelTemp {
    counter: LengthCounter,
}

impl LengthCountedChannelTemp {
    pub fn length_counter(&self) -> &LengthCounter {
        &self.counter
    }

    pub fn length_counter_mut(&mut self) -> &mut LengthCounter {
        &mut self.counter
    }
}

#[derive(Debug, Default)]
pub struct Timer {
    count: u16,
    period: u16,
}

impl Timer {
    pub fn clock(&mut self) -> bool {
        if self.count == 0 {
            self.count = self.period;
            true
        } else {
            self.count -= 1;
            false
        }
    }

    pub fn set_period(&mut self, period: u16) {
        self.period = period
    }

    fn period(&self) -> u16 {
        self.period
    }
}

#[derive(Debug, Default)]
pub struct Sequencer {
    sequence: Vec<u8>,
    position: usize,
}

impl Sequencer {
    pub fn new(sequence: &[u8]) -> Self {
        Self {
            sequence: sequence.into(),
            position: 0,
        }
    }

    pub fn set_sequence(&mut self, sequence: &[u8]) {
        self.sequence.clear();
        self.sequence.extend_from_slice(sequence);
    }

    pub fn reset(&mut self) {
        self.position = 0;
    }

    pub fn advance(&mut self) {
        self.position = self.position.wrapping_add(1) % self.length();
    }

    pub fn length(&self) -> usize {
        self.sequence.len()
    }

    pub fn value(&self) -> u8 {
        *self.sequence.get(self.position).unwrap_or(&0)
    }
}

#[derive(Debug, Default)]
pub struct SquarePulseChannel {}

#[derive(Debug, Default)]
pub struct TriangleChannel {}

#[derive(Debug, Default)]
pub struct NoiseChannel {}

#[derive(Debug, Default)]
pub struct EnvelopeGenerator {
    restart: bool,
    loop_flag: bool,
    constant_volume: bool,
    volume: u8,

    decay_level: u8,
    divider: Divider,
}

impl EnvelopeGenerator {
    pub fn clock(&mut self) {
        if self.restart {
            self.restart = false;
            self.decay_level = 15;
            self.divider.reset();
        } else if self.divider.clock() {
            if self.decay_level > 0 {
                self.decay_level -= 1;
            } else if self.loop_flag {
                self.decay_level = 15;
            }
        }
    }

    pub fn volume(&self) -> f32 {
        if self.constant_volume {
            self.volume as f32
        } else {
            self.decay_level as f32
        }
    }

    pub fn set_restart(&mut self, value: bool) {
        self.restart = value;
    }

    pub fn set_loop(&mut self, value: bool) {
        self.loop_flag = value;
    }

    pub fn set_constant_volume(&mut self, value: bool) {
        self.constant_volume = value;
    }

    pub fn set_volume(&mut self, value: u8) {
        self.volume = value;
        self.divider.set_period(value as u16);
    }
}

#[derive(Debug, Default)]
struct Divider {
    count: u16,
    period: u16,
}

impl Divider {
    fn clock(&mut self) -> bool {
        if self.count == 0 {
            self.reset();
            true
        } else {
            self.count -= 1;
            false
        }
    }

    fn reset(&mut self) {
        self.count = self.period
    }

    fn set_period(&mut self, period: u16) {
        self.period = period;
    }
}
