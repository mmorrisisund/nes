// |  0   1   2   3   4   5   6   7    8   9   A   B   C   D   E   F
// -----+----------------------------------------------------------------
// 00-0F  10,254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
// 10-1F  12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30
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
    pub fn new() -> Self {
        Self {
            counter: 0,
            enabled: false,
            halt: false,
        }
    }

    pub fn load_length_counter(&mut self, index: u8) {
        debug_assert!(index <= 0x1f);

        if self.enabled {
            self.counter = LENGTH_TABLE[index as usize];
        }
    }

    pub fn counter(&self) -> u8 {
        self.counter
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

    pub fn decrement(&mut self) {
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
pub struct LengthCountedChannel<T> {
    counter: LengthCounter,
    channel: T,
}

impl<T> LengthCountedChannel<T> {
    pub fn new(channel: T) -> Self {
        Self {
            counter: LengthCounter::new(),
            channel,
        }
    }

    pub fn length_counter(&self) -> &LengthCounter {
        &self.counter
    }

    pub fn length_counter_mut(&mut self) -> &mut LengthCounter {
        &mut self.counter
    }

    pub fn channel(&self) -> &T {
        &self.channel
    }

    pub fn channel_mut(&mut self) -> &mut T {
        &mut self.channel
    }
}

#[derive(Debug, Default)]
pub struct SquarePulseChannel {}

#[derive(Debug, Default)]
pub struct TriangleChannel {}

#[derive(Debug, Default)]
pub struct NoiseChannel {}
