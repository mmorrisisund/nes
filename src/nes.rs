use std::path::Path;

use dynwave::AudioPlayer;

use crate::{
    bus::MainBus,
    cartridge::{Cartridge, CartridgeError},
    common::mem::Reset,
    cpu::{Cpu, RunState},
};

pub struct Nes {
    cpu: Cpu,
    pub audio_player: AudioPlayer<f32>,
}

impl Nes {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, CartridgeError> {
        let cartridge = Cartridge::from_file(file_path)?;
        let mut bus = MainBus::new();
        bus.load_cart(cartridge);

        let cpu = Cpu::new(bus);

        Ok(Self {
            cpu,
            audio_player: AudioPlayer::new(44100, dynwave::BufferSize::Samples(1000000)).unwrap(),
        })
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.bus_mut().reset();
    }

    pub fn clock_frame(&mut self) {
        for _ in 0..29780 {
            self.clock_cycle();
        }
    }

    pub fn clock_cycle(&mut self) -> RunState {
        self.cpu.clock_cycle()
    }

    pub fn bus_mut(&mut self) -> &mut MainBus {
        self.cpu.bus_mut()
    }
}
