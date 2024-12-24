use crate::{
    apu::APU,
    cartridge::Cartridge,
    common::{
        bus::Bus,
        interconnect::{InterruptSignaler, SignalProvider},
    },
    cpu::{RunState, CPU},
    ppu::PPU,
};

struct TestNes {
    cpu: CPU<TestBusRom>,
}

impl TestNes {
    fn reset(&mut self) {
        self.cpu.reset();
    }

    fn cycle_until_infinite(&mut self) {
        loop {
            // println!("{}", self.cpu.trace());
            let state = self.cpu.cycle_tick();
            if let RunState::InfiniteLoop(_) = state {
                return;
            }
        }
    }

    fn cycle_until_mem_neq(&mut self, address: u16, value: u8) {
        loop {
            self.cpu.cycle_tick();

            if self.cpu.bus().read(address) != value {
                return;
            }
        }
    }

    fn cpu_bus(&self) -> &TestBusRom {
        self.cpu.bus()
    }

    fn cpu_trace(&mut self) -> String {
        self.cpu.trace()
    }
}

pub struct TestBusRom {
    ram: [u8; 0x800],
    apu: APU,
    ppu: PPU,
    cartridge: Cartridge,
}

impl TestBusRom {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            ram: [0; 0x800],
            apu: APU::default(),
            ppu: PPU {},
            cartridge,
        }
    }
}

impl Bus for TestBusRom {
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

impl SignalProvider for TestBusRom {
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

fn create_test_nes(file_path: &str) -> TestNes {
    let cartridge = Cartridge::from_file(file_path).unwrap();
    let bus = TestBusRom::new(cartridge);
    let cpu = CPU::new(bus);

    TestNes { cpu }
}

pub fn execute_blargg_test(file_path: &str) {
    let mut nes = create_test_nes(file_path);

    nes.reset();

    nes.cycle_until_infinite();
    nes.cycle_until_mem_neq(0x6000, 0x80);

    let result = nes.cpu_bus().read(0x6000);
    assert_eq!(result, 0, "\n\ntrace: {}\n", nes.cpu_trace());
}
