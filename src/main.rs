mod apu;
pub mod bus;
mod cartridge;
mod common;
mod cpu;
mod nes;
mod ppu;

use cpu::RunState;
use nes::Nes;

fn main() {
    let mut nes = Nes::new("./src/test_roms/cpu/branch_timing/1.Branch_Basics.nes").unwrap();

    nes.reset();

    let mut state = RunState::NormalExecution;
    loop {
        if let RunState::NormalExecution = state {
            println!("{}", nes.cpu_trace());
        }
        state = nes.cycle_tick();

        if let RunState::InfiniteLoop(_) = state {
            panic!("done?");
        }
    }
}
