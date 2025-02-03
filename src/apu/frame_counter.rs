use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::common::interconnect::InterruptSignaler;

#[derive(Debug, Default, PartialEq, Clone, Copy, Serialize, Deserialize)]
enum SequencerMode {
    #[default]
    FourStep,
    FiveStep,
}

impl From<u8> for SequencerMode {
    fn from(value: u8) -> Self {
        if value & 0x80 == 0 {
            Self::FourStep
        } else {
            Self::FiveStep
        }
    }
}

pub enum FrameType {
    Quarter,
    Half,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrameCounter {
    count: u16,
    count_reset_delay: i8,

    interrupt_flag: bool,
    interrupt_disabled: bool,

    sequencer_mode: SequencerMode,
    pending_sequencer_mode: SequencerMode,
}

impl FrameCounter {
    pub fn new() -> Self {
        Self {
            count: 5, // TODO: cpu has 7 cycle delay at start, must be odd!!??? or jitter test fails, if 0 both 4017 tests fail, if 1 4017 timing test fails
            count_reset_delay: -1,

            interrupt_disabled: false,
            interrupt_flag: false,
            sequencer_mode: SequencerMode::FourStep,
            pending_sequencer_mode: SequencerMode::FourStep,
        }
    }

    pub fn reset(&mut self) {
        self.count = 7;
        self.interrupt_flag = false;
    }

    pub fn set_register_value(&mut self, value: u8) {
        self.pending_sequencer_mode = value.into();
        self.interrupt_disabled = value & (1 << 6) != 0;

        if self.interrupt_disabled {
            self.interrupt_flag = false;
        }
        self.count_reset_delay = if self.count % 2 == 0 { 3 } else { 2 };
    }

    pub fn clock(&mut self, mut on_clock: impl FnMut(FrameType)) {
        self.count += 1;

        match self.count_reset_delay.cmp(&0) {
            Ordering::Less => {}
            Ordering::Equal => {
                self.count_reset_delay -= 1;

                self.count = 0;
                self.sequencer_mode = self.pending_sequencer_mode;

                if let SequencerMode::FiveStep = self.sequencer_mode {
                    on_clock(FrameType::Half);
                }
            }
            Ordering::Greater => self.count_reset_delay -= 1,
        }

        match self.count {
            7457 => {
                on_clock(FrameType::Quarter);
            }
            14913 => {
                on_clock(FrameType::Half);
            }
            22371 => {
                on_clock(FrameType::Quarter);
            }
            29828 => {
                if self.sequencer_mode == SequencerMode::FourStep {
                    self.raise_interrupt();
                }
            }
            29829 => {
                if self.sequencer_mode == SequencerMode::FourStep {
                    on_clock(FrameType::Half);

                    self.raise_interrupt();
                }
            }
            29830 if self.sequencer_mode == SequencerMode::FourStep => {
                self.raise_interrupt();

                self.count = 0;
            }
            37281 => {
                on_clock(FrameType::Half);
            }
            37282 => {
                self.count = 0;
            }
            _ => {} //ignore
        }
    }

    pub fn clear_interrupt(&mut self) {
        self.interrupt_flag = false;
    }

    fn raise_interrupt(&mut self) {
        if !self.interrupt_disabled {
            self.interrupt_flag = true;
        }
    }
}

impl InterruptSignaler for FrameCounter {
    fn interrupt_status(&self) -> bool {
        self.interrupt_flag
    }

    fn dma_status(&self) -> Option<u16> {
        None
    }
}
