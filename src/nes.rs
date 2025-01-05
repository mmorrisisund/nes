use std::path::Path;

use crate::{
    // apu::{ApuRegisters, Channel, APU},
    bus::Bus,
    cartridge::{Cartridge, CartridgeError},
    common::{
        // bus::{MemReader, MemWriter, Reset},
        bus::Reset,
        // interconnect::{InterruptSignaler, SignalProvider},
    },
    cpu::{Cpu, RunState},
    // ppu::PPU,
};

pub struct Nes {
    cpu: Cpu,
}

impl Nes {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, CartridgeError> {
        let cartridge = Cartridge::from_file(file_path)?;
        let mut bus = Bus::new();
        bus.load_cart(cartridge);

        let cpu = Cpu::new(bus);

        Ok(Self { cpu })
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.bus_mut().reset();
    }

    pub fn cycle_tick(&mut self) -> RunState {
        self.cpu.cycle_tick()
    }

    pub fn cpu_trace(&mut self) -> String {
        self.cpu.trace()
    }

    pub fn bus_mut(&mut self) -> &mut Bus {
        self.cpu.bus_mut()
    }
}
