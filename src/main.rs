use std::collections::HashMap;

use nes::common::mem::MemAccess;
use nes::controller::JoypadButton;
use nes::nes::Nes;

use nes::ppu::TOTAL_PIXELS;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

fn main() {
    run_game_rom();
    // run_test_rom();
}

#[allow(dead_code)]
fn run_game_rom() {
    let mut nes = Nes::new("./game_roms/pacman.nes").unwrap();

    nes.reset();

    let mut key_map = HashMap::new();
    key_map.insert(Keycode::A, JoypadButton::A);
    key_map.insert(Keycode::B, JoypadButton::B);
    key_map.insert(Keycode::Space, JoypadButton::Select);
    key_map.insert(Keycode::Return, JoypadButton::Start);
    key_map.insert(Keycode::Up, JoypadButton::Up);
    key_map.insert(Keycode::Down, JoypadButton::Down);
    key_map.insert(Keycode::Left, JoypadButton::Left);
    key_map.insert(Keycode::Right, JoypadButton::Right);

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

    // let dbg_window = video_subsystem
    //     // .window("debug", (256 * 3) as u32, (240 * 3) as u32)
    //     .window("debug", (64 * 3) as u32, (64 * 3) as u32)
    //     .position_centered()
    //     .build()
    //     .unwrap();
    // let mut dbg_canvas = dbg_window.into_canvas().present_vsync().build().unwrap();
    // dbg_canvas.set_scale(3.0, 3.0).unwrap();
    // let dbg_creator = dbg_canvas.texture_creator();
    // let mut dbg_texture = dbg_creator
    //     // .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
    //     .create_texture_target(PixelFormatEnum::RGB24, 64, 64)
    //     .unwrap();

    let mut paused = false;
    loop {
        if !paused {
            nes.clock_frame();
        }
        // let bus = nes.bus_mut();
        let frame = nes.bus_mut().frame_buffer();

        // let mut dbg_frame = vec![0x0; ppu::TOTAL_PIXELS * 2 * 3]; // 2 pattern tables, 3 channels RGB
        // bus.ppu.load_pattern_tables(&mut dbg_frame);
        // dbg_texture.update(None, &dbg_frame, 256 * 3).unwrap();
        // let mut dbg_frame = vec![0x0; ppu::TOTAL_PIXELS * 4 * 3]; // 4 nametables, 3 channels RGB
        // bus.ppu.load_name_tables(&mut dbg_frame);
        // dbg_texture.update(None, &dbg_frame, 256 * 3).unwrap();
        // let mut dbg_frame = vec![0x0; TOTAL_PIXELS * 3]; // palette
        // bus.ppu.load_palette(&mut dbg_frame);
        // dbg_texture.update(None, &dbg_frame, 256 * 3).unwrap();
        // let mut dbg_frame = vec![0x00; 64 * 8 * 8 * 3]; // sprites
        // bus.ppu.load_oam(&mut dbg_frame);
        // dbg_texture.update(None, &dbg_frame, 64 * 3).unwrap();
        // dbg_canvas.copy(&dbg_texture, None, None).unwrap();
        // dbg_canvas.present();

        texture.update(None, frame, 256 * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        // let joypad = &mut bus.joypad1;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::ESCAPE),
                    ..
                } => std::process::exit(0),

                Event::KeyDown { keycode, .. } => match keycode.unwrap() {
                    Keycode::D => nes.save_state(),
                    Keycode::F => nes.clock_frame(),
                    Keycode::L => nes.load_state(),
                    Keycode::P => paused = !paused,
                    _ => {
                        if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                            nes.bus_mut().joypad1.set_button(*key, true);
                        }
                    }
                },
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        nes.bus_mut().joypad1.set_button(*key, false);
                    }
                }
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
