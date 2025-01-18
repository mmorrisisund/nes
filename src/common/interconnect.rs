pub trait InterruptSignaler {
    fn interrupt_status(&self) -> bool;
    fn dma_status(&self) -> Option<u16>;
}

pub trait SignalProvider {
    fn interrupt_requested(&self) -> Option<Interrupt>;
    fn dma_requested(&self) -> Option<DmaSource>;
}

#[derive(Debug, PartialEq)]
pub enum DmaSource {
    Apu(u16),
    Ppu(u8),
}

#[derive(Debug, PartialEq)]
pub enum Interrupt {
    Irq,
    Nmi,
    Break,
}

impl Interrupt {
    const IRQ_VECTOR_ADDRESS: u16 = 0xfffe;
    const NMI_VECTOR_ADDRESS: u16 = 0xfffa;

    pub fn vector(&self) -> u16 {
        match self {
            Self::Irq => Self::IRQ_VECTOR_ADDRESS,
            Self::Nmi => Self::NMI_VECTOR_ADDRESS,
            Self::Break => Self::IRQ_VECTOR_ADDRESS,
        }
    }
}
