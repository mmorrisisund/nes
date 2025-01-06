use crate::{
    apu::Apu,
    bus::Bus,
    cartridge::Cartridge,
    common::{
        bus::{MemAccess, Reset},
        interconnect::{InterruptSignaler, SignalProvider},
    },
    cpu::RunState,
    nes::Nes,
    ppu::Ppu,
};

const STATUS_ADDR: u16 = 0x6000;
const TEXT_ADDR: u16 = 0x6004;
const TEST_RUNNING: u8 = 0x80;
const RESET_NEEDED: u8 = 0x81;

struct TestNes {
    nes: Nes,
    // cpu: CPU<TestBusRom>,
}

impl TestNes {
    fn reset(&mut self) {
        self.nes.reset();
    }

    fn cycle_until_infinite(&mut self) -> u64 {
        // reminder: don't trace, alters apu register on read
        // println!("{}", self.nes.cpu_trace());
        let mut total_cycles = 0;
        loop {
            total_cycles += 1;
            let state = self.nes.clock_cycle();
            if let RunState::NormalExecution = state {
                // println!("{}", self.nes.cpu_trace());
            }
            if let RunState::InfiniteLoop(_) = state {
                return total_cycles;
            }
        }
    }

    fn cycle_until_mem_neq(&mut self, address: u16, value: u8) {
        // reminder: don't trace, alters apu register on read
        loop {
            self.nes.clock_cycle();

            if self.nes.bus_mut().read(address) != value {
                return;
            }
        }
    }

    fn cpu_bus_mut(&mut self) -> &mut Bus {
        self.nes.bus_mut()
    }
}

pub struct TestBusRom {
    ram: [u8; 0x800],
    apu: Apu,
    ppu: Ppu,
    cartridge: Cartridge,
}

impl MemAccess for TestBusRom {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..0x2000 => self.ram[(address & 0x7ff) as usize], // ram
            0x2000..0x4000 => 0,                                    // ppu
            0x4000..0x4018 => 0,                                    // apu
            0x4018..0x4020 => 0,                                    // unused
            0x4020.. => self.cartridge.read(address),               // cartridge
        }
    }

    fn peek(&self, _address: u16) -> u8 {
        0
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

impl Reset for TestBusRom {
    fn reset(&mut self) {
        self.apu.reset();
    }
}

impl SignalProvider for TestBusRom {
    fn nmi_requested(&self) -> bool {
        self.ppu.interrupt_status()
    }

    fn clear_nmi(&mut self) {
        self.ppu.clear_interrupt();
    }

    fn interrupt_status(&self) -> bool {
        self.apu.interrupt_status()
    }

    fn interrupt_status_changed(&self) -> bool {
        self.apu.interrupt_status_changed()
    }

    fn clear_interrupt(&mut self) {
        self.apu.clear_interrupt();
    }
}

fn create_test_nes(file_path: &str) -> TestNes {
    let nes = Nes::new(file_path).unwrap();
    TestNes { nes }
}

pub fn execute_blargg_test(file_path: &str) {
    let mut nes = create_test_nes(file_path);

    // nes.reset();

    nes.cycle_until_infinite();
    nes.cycle_until_mem_neq(STATUS_ADDR, TEST_RUNNING);

    // let mut status = ;
    while nes.cpu_bus_mut().read(STATUS_ADDR) == RESET_NEEDED {
        println!("reset performed");
        nes.reset();

        nes.cycle_until_infinite();
        nes.cycle_until_mem_neq(STATUS_ADDR, TEST_RUNNING);
    }

    let result = nes.cpu_bus_mut().read(STATUS_ADDR);
    let text = get_text_output(&mut nes);
    assert_eq!(result, 0, "\n{}", text);
}

fn get_text_output(nes: &mut TestNes) -> String {
    let mut addr = TEXT_ADDR;
    let mut chars = Vec::new();
    let mut c = nes.cpu_bus_mut().read(addr);

    while c != 0 {
        chars.push(c);

        addr += 1;
        c = nes.cpu_bus_mut().read(addr)
    }

    if !chars.is_empty() {
        String::from_utf8_lossy(&chars).to_string()
    } else {
        String::new()
    }
}
