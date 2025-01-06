mod apu;
pub mod bus;
mod cartridge;
mod common;
mod cpu;
mod nes;
mod ppu;

use common::bus::MemAccess;
// use cpu::RunState;
use nes::Nes;

// fn main() {
//     use dynwave::{AudioPlayer, BufferSize};
//     let sample_rate = 44100;
//     let buffer_size = BufferSize::Samples(100000);
//     let mut player = AudioPlayer::new(sample_rate, buffer_size).unwrap();

//     let samples = vec![0.5, 0.7, 0.9, 1.0, 0.9, 0.7, 0.5, 0.3, 0.1];
//     player.play().unwrap();
//     for _ in 0..1000000 {
//         player.queue(&samples);
//     }
// }

fn main() {
    let mut nes = Nes::new("./src/test_roms/apu/mixer/square.nes").unwrap();

    nes.reset();

    // let mut state = RunState::NormalExecution;
    let start = std::time::Instant::now();
    let run_time = std::time::Duration::from_secs(10);

    const TEXT_ADDR: u16 = 0x6004;
    let mut addr = TEXT_ADDR;
    let mut chars = Vec::new();
    let mut c = nes.bus_mut().read(addr);
    let mut check;

    let mut cycles = 0;
    nes.audio_player.play().unwrap();

    loop {
        cycles += 1;
        // nes.clock_frame();
        nes.clock_cycle();

        if cycles % (29780) == 0 {
            let samples = nes.bus_mut().drain_audio_buffer();
            nes.audio_player.queue(&samples);
        }

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

    // loop {
    //     std::thread::sleep(std::time::Duration::from_millis(10));
    //     // if let RunState::NormalExecution = state {
    //     //     println!("{}", nes.cpu_trace());
    //     // }
    //     // state = nes.clock_cycle();

    //     // if let RunState::InfiniteLoop(_) = state {
    //     //     panic!("done?");
    //     // }
    // }
}
