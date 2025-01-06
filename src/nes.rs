use std::path::Path;

use dynwave::AudioPlayer;

use crate::{
    // apu::{ApuRegisters, Channel, APU},
    bus::Bus,
    cartridge::{Cartridge, CartridgeError},
    common::{
        // bus::{MemReader, MemWriter, Reset},
        bus::Reset,
        // interconnect::{InterruptSignaler, SignalProvider},
    },
    cpu::{Cpu, RunState},
    // ppu::PPU,
};

pub struct Nes {
    cpu: Cpu,
    pub audio_player: AudioPlayer<f32>,
}

impl Nes {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, CartridgeError> {
        let cartridge = Cartridge::from_file(file_path)?;
        let mut bus = Bus::new();
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

        let samples = self.bus_mut().drain_audio_buffer();
        self.audio_player.queue(&samples);

        self.audio_player.play().unwrap();
    }

    pub fn clock_cycle(&mut self) -> RunState {
        self.cpu.clock_cycle()
    }

    pub fn cpu_trace(&mut self) -> String {
        self.cpu.trace()
    }

    pub fn bus_mut(&mut self) -> &mut Bus {
        self.cpu.bus_mut()
    }
}
