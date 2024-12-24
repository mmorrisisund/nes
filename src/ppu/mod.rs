use crate::common::interconnect::InterruptSignaler;

pub struct PPU {}

impl InterruptSignaler for PPU {
    fn interrupt_status(&self) -> bool {
        false
    }

    fn clear_interrupt(&mut self) {}
}
