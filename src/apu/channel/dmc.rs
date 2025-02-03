use serde::{Deserialize, Serialize};

use super::{ApuChannel, Timer, TimerTrait};
use crate::common::interconnect::InterruptSignaler;

const PERIOD_LENGTH: [usize; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Dmc {
    timer: Timer,
    sample_buffer: u8,
    shift: u8,

    buffer_empty: bool,

    base_sample_addr: u16,
    current_sample_addr: u16,
    sample_length: u16,
    loop_sample: bool,
    bytes_remaining: u16,

    bits_remaining: u8,
    output_level: u8,
    silence: bool,

    irq_enabled: bool,
    interrupt_flag: bool, // irq with info that entire sample exhaustd. all bytes read.
                          // dma request is just for next byte when there are still bytes left
}

impl Dmc {
    pub fn new() -> Self {
        Self {
            buffer_empty: true,
            bits_remaining: 8,
            timer: Timer::new(0, 1),
            ..Default::default()
        }
    }

    pub fn write_timer(&mut self, value: u8) {
        self.irq_enabled = value & 0x80 == 0x80;
        self.loop_sample = value & 0x40 == 0x40;
        self.timer
            .set_period(PERIOD_LENGTH[(value & 0xf) as usize] - 1); // TODO: WTF??!!! the timer runs for for P+1, which works for all other units.  Wasn't working for this one.  why????

        if !self.irq_enabled {
            self.interrupt_flag = false;
        }
    }

    pub fn set_output(&mut self, value: u8) {
        self.output_level = value & 0x7f;
    }

    pub fn set_address(&mut self, value: u8) {
        self.base_sample_addr = 0xc000 | (u16::from(value) << 6);
    }

    pub fn set_length(&mut self, value: u8) {
        self.sample_length = (u16::from(value) << 4) | 1;
    }

    pub fn clear_interrupt(&mut self) {
        self.interrupt_flag = false;
    }

    pub fn set_sample_buffer(&mut self, value: u8) {
        self.sample_buffer = value;
        self.buffer_empty = false;

        if self.current_sample_addr == 0xffff {
            self.current_sample_addr = 0x8000;
        } else {
            self.current_sample_addr += 1;
        }
        self.bytes_remaining = self.bytes_remaining.saturating_sub(1);

        if self.bytes_remaining == 0 {
            if self.loop_sample {
                self.current_sample_addr = self.base_sample_addr;
                self.bytes_remaining = self.sample_length;
            } else if self.irq_enabled {
                self.interrupt_flag = true;
            }
        }
    }
}

impl ApuChannel for Dmc {
    fn enabled(&self) -> bool {
        self.bytes_remaining > 0
    }

    fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.bytes_remaining = 0;
        } else if self.bytes_remaining == 0 {
            self.current_sample_addr = self.base_sample_addr;
            self.bytes_remaining = self.sample_length;
        }
    }

    fn get_output(&self) -> f32 {
        self.output_level as f32
    }
}

impl TimerTrait for Dmc {
    fn clock(&mut self) {
        if self.timer.clock() {
            if !self.silence {
                if self.shift & 0x1 == 1 && self.output_level <= 125 {
                    self.output_level += 2;
                } else if self.shift & 0x1 == 0 && self.output_level >= 2 {
                    self.output_level -= 2;
                }
            }

            self.shift >>= 1;
            self.bits_remaining = self.bits_remaining.saturating_sub(1);

            if self.bits_remaining == 0 {
                self.bits_remaining = 8;

                self.silence = self.buffer_empty;

                if !self.buffer_empty {
                    self.shift = self.sample_buffer;
                    self.buffer_empty = true;
                }
            }
        }
    }
}

impl InterruptSignaler for Dmc {
    fn interrupt_status(&self) -> bool {
        self.interrupt_flag
    }

    fn dma_status(&self) -> Option<u16> {
        if self.buffer_empty && self.bytes_remaining > 0 {
            Some(self.current_sample_addr)
        } else {
            None
        }
    }
}
