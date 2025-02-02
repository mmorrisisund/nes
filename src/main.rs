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

use ppu::TOTAL_PIXELS;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

fn main() {
    // let cartridge = cartridge::Cartridge::from_file("./game_roms/pacman.nes").unwrap();

    // let cartridge =
    //     cartridge::Cartridge::from_file("./src/test_roms/cpu/instr_timing/instr_timing.nes")
    //         .unwrap();
    // println!("{cartridge:?}");

    run_game_rom();
    // run_test_rom();
}

#[allow(dead_code)]
fn run_game_rom() {
    let mut nes = Nes::new("./game_roms/pacman.nes").unwrap();

    nes.reset();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("NES", (256 * 3) as u32, (240 * 3) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let dbg_window = video_subsystem
        // .window("debug", (256 * 3) as u32, (240 * 3) as u32)
        .window("debug", (64 * 3) as u32, (64 * 3) as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut dbg_canvas = dbg_window.into_canvas().present_vsync().build().unwrap();
    dbg_canvas.set_scale(3.0, 3.0).unwrap();
    let dbg_creator = dbg_canvas.texture_creator();
    let mut dbg_texture = dbg_creator
        // .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .create_texture_target(PixelFormatEnum::RGB24, 64, 64)
        .unwrap();

    loop {
        nes.clock_frame();
        let bus = nes.bus_mut();
        let frame = bus.frame_buffer();

        // let mut dbg_frame = vec![0x0; ppu::TOTAL_PIXELS * 2 * 3]; // 2 pattern tables, 3 channels RGB
        // bus.ppu.load_pattern_tables(&mut dbg_frame);
        // dbg_texture.update(None, &dbg_frame, 256 * 3).unwrap();

        // let mut dbg_frame = vec![0x0; ppu::TOTAL_PIXELS * 4 * 3]; // 4 nametables, 3 channels RGB
        // bus.ppu.load_name_tables(&mut dbg_frame);
        // dbg_texture.update(None, &dbg_frame, 256 * 3).unwrap();

        // let mut dbg_frame = vec![0x0; TOTAL_PIXELS * 3]; // palette
        // bus.ppu.load_palette(&mut dbg_frame);
        // dbg_texture.update(None, &dbg_frame, 256 * 3).unwrap();

        let mut dbg_frame = vec![0x00; 64 * 8 * 8 * 3]; // sprites
        bus.ppu.load_oam(&mut dbg_frame);
        dbg_texture.update(None, &dbg_frame, 64 * 3).unwrap();

        dbg_canvas.copy(&dbg_texture, None, None).unwrap();
        dbg_canvas.present();

        texture.update(None, frame, 256 * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::ESCAPE),
                    ..
                } => std::process::exit(0),
                _ => {}
            }
        }
    }
}

#[allow(dead_code)]
fn run_test_rom() {
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
#[rustfmt::skip]

pub static SYSTEM_PALETTE: [(u8,u8,u8); 64] = [
   (0x80, 0x80, 0x80), (0x00, 0x3d, 0xa6), (0x00, 0x12, 0xb0), (0x44, 0x00, 0x96), (0xa1, 0x00, 0x5e),
   (0xc7, 0x00, 0x28), (0xba, 0x06, 0x00), (0x8c, 0x17, 0x00), (0x5c, 0x2f, 0x00), (0x10, 0x45, 0x00),
   (0x05, 0x4a, 0x00), (0x00, 0x47, 0x2e), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05),
   (0x05, 0x05, 0x05), (0xc7, 0xc7, 0xc7), (0x00, 0x77, 0xff), (0x21, 0x55, 0xff), (0x82, 0x37, 0xfa),
   (0xeb, 0x2f, 0xb5), (0xff, 0x29, 0x50), (0xff, 0x22, 0x00), (0xd6, 0x32, 0x00), (0xc4, 0x62, 0x00),
   (0x35, 0x80, 0x00), (0x05, 0x8f, 0x00), (0x00, 0x8a, 0x55), (0x00, 0x99, 0xcc), (0x21, 0x21, 0x21),
   (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xff, 0xff, 0xff), (0x0f, 0xd7, 0xff), (0x69, 0xa2, 0xff),
   (0xd4, 0x80, 0xff), (0xff, 0x45, 0xf3), (0xff, 0x61, 0x8b), (0xff, 0x88, 0x33), (0xff, 0x9c, 0x12),
   (0xfa, 0xbc, 0x20), (0x9f, 0xe3, 0x0e), (0x2b, 0xf0, 0x35), (0x0c, 0xf0, 0xa4), (0x05, 0xfb, 0xff),
   (0x5e, 0x5e, 0x5e), (0x0d, 0x0d, 0x0d), (0x0d, 0x0d, 0x0d), (0xff, 0xff, 0xff), (0xa6, 0xfc, 0xff),
   (0xb3, 0xec, 0xff), (0xda, 0xab, 0xeb), (0xff, 0xa8, 0xf9), (0xff, 0xab, 0xb3), (0xff, 0xd2, 0xb0),
   (0xff, 0xef, 0xa6), (0xff, 0xf7, 0x9c), (0xd7, 0xe8, 0x95), (0xa6, 0xed, 0xaf), (0xa2, 0xf2, 0xda),
   (0x99, 0xff, 0xfc), (0xdd, 0xdd, 0xdd), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11)
];
