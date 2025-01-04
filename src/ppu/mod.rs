use crate::common::interconnect::InterruptSignaler;

pub struct Ppu {}

impl InterruptSignaler for Ppu {
    fn interrupt_status(&self) -> bool {
        false
    }

    fn interrupt_status_changed(&self) -> bool {
        todo!()
    }

    fn clear_interrupt(&mut self) {}
}
