use common::Bus;
use cpu::{RunState, CPU};

pub mod common;
pub mod cpu;

fn main() {
    let file_data = include_bytes!(".\\test roms\\nestest.nes");
    let mut data = [0; 0x10000];

    let rom_data = &file_data[0x10..0x4010];
    data[0x8000..0xc000].clone_from_slice(rom_data);
    data[0xc000..].clone_from_slice(rom_data);

    data[0xfffc] = 0x00;
    data[0xfffd] = 0xc0;

    let bus = TestBus { data };
    let mut cpu = CPU::new(bus);

    const SUCCESS_ADDRESS: u16 = 0x336d;

    cpu.reset();
    loop {
        let state = cpu.run_next_instruction();

        // if we stuck in a loop, return error
        if let RunState::InfiniteLoop(pc) = state {
            assert!(
                pc == SUCCESS_ADDRESS,
                "Test failed at {:04X}, check the `.lst` file for more info",
                pc
            );
            break;
        }
    }
}
struct TestBus {
    data: [u8; 0x10000],
}

impl Bus for TestBus {
    fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0002 | 0x0003 => println!("write: {data:02X}"),
            _ => {}
        }
        self.data[address as usize] = data;
    }
}
