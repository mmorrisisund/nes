pub trait InterruptSignaler {
    fn interrupt_status(&self) -> bool;
    fn interrupt_status_changed(&self) -> bool;
    fn clear_interrupt(&mut self);
}

pub trait SignalProvider {
    fn nmi_requested(&self) -> bool;
    fn clear_nmi(&mut self);

    fn interrupt_status(&self) -> bool;
    fn interrupt_status_changed(&self) -> bool;
    fn clear_interrupt(&mut self);
}
