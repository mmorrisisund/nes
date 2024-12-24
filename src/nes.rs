use std::path::Path;

use crate::{
    apu::APU,
    cartridge::{Cartridge, CartridgeError},
    common::{
        bus::Bus,
        interconnect::{InterruptSignaler, SignalProvider},
    },
    cpu::{RunState, CPU},
    ppu::PPU,
};

struct MainBus {
    ram: [u8; 0x800],
    apu: APU,
    ppu: PPU,
    cartridge: Cartridge,
}

impl MainBus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            ram: [0; 0x800],
            apu: APU {},
            ppu: PPU {},
            cartridge,
        }
    }
}

impl Bus for MainBus {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..0x2000 => self.ram[(address & 0x7ff) as usize], // ram
            0x2000..0x4000 => 0,                                    // ppu
            0x4000..0x4018 => 0,                                    // apu
            0x4018..0x4020 => 0,                                    // unused
            0x4020.. => self.cartridge.read(address),               // cartridge
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..0x2000 => self.ram[(address & 0x7ff) as usize] = data, // ram
            0x2000..0x4000 => {}                                           // ppu
            0x4000..0x4018 => {}                                           // apu
            0x4018..0x4020 => {}                                           // unused
            0x4020.. => self.cartridge.write(address, data),               // cartridge
        }
    }
}

impl SignalProvider for MainBus {
    fn nmi_requested(&self) -> bool {
        self.ppu.interrupt_status()
    }

    fn clear_nmi(&mut self) {
        self.ppu.clear_interrupt();
    }

    fn irq_requested(&self) -> bool {
        self.apu.interrupt_status()
    }

    fn clear_irq(&mut self) {
        self.apu.clear_interrupt();
    }
}

pub struct Nes {
    cpu: CPU<MainBus>,
}

impl Nes {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, CartridgeError> {
        let cartridge = Cartridge::from_file(file_path)?;
        let bus = MainBus::new(cartridge);
        let cpu = CPU::new(bus);

        Ok(Self { cpu })
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    pub fn cycle_tick(&mut self) -> RunState {
        self.cpu.cycle_tick()
    }

    pub fn cpu_trace(&mut self) -> String {
        self.cpu.trace()
    }
}
