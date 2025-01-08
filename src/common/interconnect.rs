pub trait InterruptSignaler {
    fn interrupt_status(&self) -> bool;
}

pub trait SignalProvider {
    fn nmi_requested(&self) -> bool;

    fn interrupt_requested(&self) -> bool;
}
