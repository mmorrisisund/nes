use std::collections::HashMap;

use nes::common::mem::MemAccess;
use nes::controller::JoypadButton;
use nes::gui::Gui;
use nes::nes::Nes;

use dynwave::AudioPlayer;
use eframe::egui::{self, CentralPanel, ColorImage, TopBottomPanel, ViewportBuilder, ViewportId};
use eframe::CreationContext;
use egui::Key;

fn main() {
    run_game_rom();
    // run_test_rom();
}

fn get_key_map() -> HashMap<Key, JoypadButton> {
    let mut key_map = HashMap::new();
    key_map.insert(Key::A, JoypadButton::A);
    key_map.insert(Key::B, JoypadButton::B);
    key_map.insert(Key::Space, JoypadButton::Select);
    key_map.insert(Key::Enter, JoypadButton::Start);
    key_map.insert(Key::ArrowUp, JoypadButton::Up);
    key_map.insert(Key::ArrowDown, JoypadButton::Down);
    key_map.insert(Key::ArrowLeft, JoypadButton::Left);
    key_map.insert(Key::ArrowRight, JoypadButton::Right);

    key_map
}

struct App {
    gui: Gui,
    nes: Nes,
    image_texture: egui::TextureHandle,
    audio_player: AudioPlayer<f32>,
    key_map: HashMap<Key, JoypadButton>,

    show_tools: bool,
    current_tab: u8,
}

impl App {
    fn new(cc: &CreationContext) -> Self {
        let mut nes = Nes::new("./game_roms/smb1.nes").unwrap();
        nes.reset();

        let image_texture = cc.egui_ctx.load_texture(
            "nes-image",
            egui::ColorImage::from_rgb([256, 240], vec![0; 256 * 240 * 3].as_slice()),
            egui::TextureOptions {
                magnification: egui::TextureFilter::Nearest,
                minification: egui::TextureFilter::Nearest,
                ..Default::default()
            },
        );

        Self {
            gui: Gui::new(),
            nes,
            image_texture,
            audio_player: AudioPlayer::new(44100, dynwave::BufferSize::QuarterSecond).unwrap(),
            key_map: get_key_map(),

            show_tools: false,
            current_tab: 0,
        }
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            for (key, button) in &self.key_map {
                self.nes
                    .bus_mut()
                    .joypad1
                    .set_button(*button, i.key_down(*key));
            }
        })
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.handle_input(ctx);

        self.nes.clock_frame();
        let bus = self.nes.bus_mut();
        let audio_buffer = bus.drain_audio_buffer();
        let frame = bus.frame_buffer();

        self.audio_player.queue(&audio_buffer);
        // self.audio_player.play().unwrap();

        self.gui.ui(ctx);

        ctx.request_repaint();
    }
}

#[allow(dead_code)]
fn run_game_rom() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([256.0 * 3., 240.0 * 3.])
            .with_min_inner_size([256.0, 240.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "nes",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    );
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
