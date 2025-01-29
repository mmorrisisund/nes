use crate::{
    apu::Apu,
    bus::MainBus,
    cartridge::Cartridge,
    common::{
        interconnect::{InterruptSignaler, SignalProvider},
        mem::{MemAccess, Reset},
    },
    cpu::RunState,
    nes::Nes,
};

use super::interconnect::Interrupt;

const STATUS_ADDR: u16 = 0x6000;
const TEXT_ADDR: u16 = 0x6004;
const TEST_RUNNING: u8 = 0x80;
const RESET_NEEDED: u8 = 0x81;

struct TestNes {
    nes: Nes,
}

impl TestNes {
    fn reset(&mut self) {
        self.nes.reset();
    }

    fn cycle_until_infinite(&mut self) -> u64 {
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
        loop {
            self.nes.clock_cycle();

            if self.nes.bus_mut().read(address) != value {
                return;
            }
        }
    }

    fn cpu_bus_mut(&mut self) -> &mut MainBus {
        self.nes.bus_mut()
    }
}

pub struct TestBusRom {
    ram: [u8; 0x800],
    apu: Apu,
    cartridge: Cartridge,
}

// impl TestBusRom {
//     pub fn new(cartridge: Cartridge) -> Self {
//         Self {
//             ram: [0; 0x800],
//             apu: Apu::new(),
//             cartridge,
//         }
//     }
// }

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
    fn interrupt_requested(&self) -> Option<Interrupt> {
        if self.apu.interrupt_status() {
            Some(Interrupt::Irq)
        } else {
            None
        }
    }

    fn dma_requested(&mut self) -> Option<super::interconnect::DmaSource> {
        todo!()
    }
}

#[macro_export]
macro_rules! blargg_test {
    ($name:ident, $root:expr, $folder:expr, $file:expr) => {
        #[test]
        fn $name() {
            $crate::common::test_helpers::execute_blargg_test(
                format!("{}{}{}", $root, $folder, $file).as_str(),
            );
        }
    };
}

fn create_test_nes(file_path: &str) -> TestNes {
    println!("{file_path}");
    let nes = Nes::new(file_path).unwrap();
    TestNes { nes }
}

pub fn execute_blargg_test(file_path: &str) {
    let mut nes = create_test_nes(file_path);

    nes.cycle_until_infinite();
    nes.cycle_until_mem_neq(STATUS_ADDR, TEST_RUNNING);

    while nes.cpu_bus_mut().read(STATUS_ADDR) == RESET_NEEDED {
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
