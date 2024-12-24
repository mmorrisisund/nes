use crate::common::interconnect::InterruptSignaler;

pub struct APU {}

impl InterruptSignaler for APU {
    fn interrupt_status(&self) -> bool {
        false
    }

    fn clear_interrupt(&mut self) {}
}
