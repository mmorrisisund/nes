use crate::common::interconnect::InterruptSignaler;

pub struct Ppu {}

impl InterruptSignaler for Ppu {
    fn interrupt_status(&self) -> bool {
        false
    }

    fn clear_interrupt(&mut self) {}
}
