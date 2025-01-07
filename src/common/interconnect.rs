pub trait InterruptSignaler {
    fn interrupt_status(&self) -> bool;
    fn clear_interrupt(&mut self);
}

pub trait SignalProvider {
    fn nmi_requested(&self) -> bool;
    fn clear_nmi(&mut self);

    fn interrupt_requested(&self) -> bool;
    fn clear_interrupt(&mut self);
}
