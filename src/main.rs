pub mod cartridge;
pub mod common;
pub mod cpu;
pub mod nes;

// use nes::Nes;

fn main() {
    // let mut nes = Nes::new("./src/test_roms/cpu/instr_timing/instr_timing.nes").unwrap();

    // nes.reset();

    // nes.cycle_until_infinite();

    // nes.cycle_until_mem_neq(0x6000, 0x80);
    // let result = nes.bus_read(0x6000);

    // println!("done: {result:04X}");
    // println!("{}", nes.cpu_trace());
}
