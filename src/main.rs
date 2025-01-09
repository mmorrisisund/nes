mod apu;
pub mod bus;
mod cartridge;
mod common;
mod cpu;
mod nes;
mod ppu;
mod util;

use common::mem::MemAccess;
use nes::Nes;

fn main() {
    let mut nes = Nes::new("./src/test_roms/cpu/instr_timing/instr_timing.nes").unwrap();

    // crate::common::test_helpers::execute_blargg_test("./src/test_roms/apu/reset/4015_cleared.nes");

    nes.reset();

    let start = std::time::Instant::now();
    let run_time = std::time::Duration::from_secs(10);

    const TEXT_ADDR: u16 = 0x6004;
    let mut addr = TEXT_ADDR;
    let mut chars = Vec::new();
    let mut c = nes.bus_mut().read(addr);
    let mut check;

    nes.audio_player.play().unwrap();

    loop {
        nes.clock_frame();

        let samples = nes.bus_mut().drain_audio_buffer();
        nes.audio_player.queue(&samples);

        if (std::time::Instant::now() - start) > run_time {
            break;
        }

        check = nes.bus_mut().read(addr);
        if check != 0 && c == 0 {
            c = check;
        }
        while c != 0 {
            chars.push(c);

            addr += 1;
            c = nes.bus_mut().read(addr)
        }

        let out = if !chars.is_empty() {
            String::from_utf8_lossy(&chars).to_string()
        } else {
            String::new()
        };
        chars.clear();
        print!("{out}");
    }
}
