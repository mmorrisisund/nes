use crate::{
    apu::{Apu, ApuRegisters, Channel},
    cartridge::Cartridge,
    common::{
        interconnect::{DmaSource, Interrupt, InterruptSignaler, SignalProvider},
        mem::{MemAccess, Reset},
    },
    ppu::Ppu,
};

pub struct MainBus {
    ram: [u8; 0x800],
    apu: Apu,
    ppu: Ppu,
    cartridge: Cartridge,
}

impl MainBus {
    pub fn new() -> Self {
        Self {
            ram: [0; 0x800],
            apu: Apu::new(),
            ppu: Ppu::default(),
            cartridge: Cartridge::empty(),
        }
    }

    pub fn load_cart(&mut self, cartridge: Cartridge) {
        self.cartridge = cartridge;
    }

    pub fn clock(&mut self) {
        self.apu.clock_cycle();

        // for _ in 0..3 {
        //     self.ppu.clock();
        // }
    }

    pub fn write_dma(&mut self, value: u8, source: DmaSource) {
        match source {
            DmaSource::Apu(_) => self.apu.dmc_write(value),
            DmaSource::Ppu(_) => {}
        }
    }

    pub fn drain_audio_buffer(&mut self) -> Vec<f32> {
        self.apu.drain_audio_buffer()
    }
}

impl Default for MainBus {
    fn default() -> Self {
        Self::new()
    }
}

impl MemAccess for MainBus {
    fn peek(&self, address: u16) -> u8 {
        match address {
            0x0000..0x2000 => self.ram[(address & 0x7ff) as usize], // ram
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 => 0, // write only registers.  should probably implement an open bus value
            0x2002 => self.ppu.peek_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.peek_ppu_data(),
            0x2008..0x4000 => self.peek(address & 0x2007), // ppu mirrors
            0x4000..0x4014 | 0x4017 => 0,
            0x4015 => self.apu.peek_status(),
            0x4014 => 0,                              // ppu
            0x4016 => 0,                              // controller
            0x4018..0x4020 => 0,                      // unused (test mode)
            0x4020.. => self.cartridge.peek(address), // cartridge
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..0x2000 => self.ram[(address & 0x7ff) as usize], // ram
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 => 0, // write only registers.  should probably implement and open bus value
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read_ppu_data(),
            0x2008..0x4000 => self.read(address & 0x2007), // ppu mirrors
            0x4000..0x4014 | 0x4017 => 0,
            0x4015 => self.apu.read_status(),
            0x4014 => 0,                              // ppu
            0x4016 => 0,                              // controller
            0x4018..0x4020 => 0,                      // unused (test mode)
            0x4020.. => self.cartridge.read(address), // cartridge
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..0x2000 => self.ram[(address & 0x7ff) as usize] = value, // ram
            0x2000 => self.ppu.write_control(value),
            0x2001 => self.ppu.write_mask(value),
            0x2002 => {} // read only register
            0x2003 => self.ppu.write_oam_addr(value),
            0x2004 => self.ppu.write_oam_data(value),
            0x2005 => self.ppu.write_scroll(value),
            0x2006 => self.ppu.write_ppu_addr(value),
            0x2007 => self.ppu.write_ppu_data(value),
            0x2008..0x4000 => self.write(address & 0x2007, value), // ppu mirrors                                         // ppu
            0x4000 => self.apu.write_ctrl(Channel::Pulse1, value),
            0x4001 => self.apu.write_sweep(Channel::Pulse1, value),
            0x4002 => self.apu.write_timer_lo(Channel::Pulse1, value),
            0x4003 => self.apu.write_timer_hi(Channel::Pulse1, value),
            0x4004 => self.apu.write_ctrl(Channel::Pulse2, value),
            0x4005 => self.apu.write_sweep(Channel::Pulse2, value),
            0x4006 => self.apu.write_timer_lo(Channel::Pulse2, value),
            0x4007 => self.apu.write_timer_hi(Channel::Pulse2, value),
            0x4008 => self.apu.write_ctrl(Channel::Triangle, value),
            0x4009 => {} // unused
            0x400a => self.apu.write_timer_lo(Channel::Triangle, value),
            0x400b => self.apu.write_timer_hi(Channel::Triangle, value),
            0x400c => self.apu.write_ctrl(Channel::Noise, value),
            0x400d => {} // unused
            0x400e => self.apu.write_timer_lo(Channel::Noise, value),
            0x400f => self.apu.write_length(Channel::Noise, value),
            0x4010 => self.apu.write_timer_lo(Channel::Dmc, value),
            0x4011 => self.apu.write_dmc_output(value),
            0x4012 => self.apu.write_dmc_addr(value),
            0x4013 => self.apu.write_length(Channel::Dmc, value),
            0x4015 => self.apu.write_status(value),
            0x4017 => self.apu.write_frame_counter(value),
            0x4014 => {}                                      // ppu
            0x4016 => {}                                      // controller
            0x4018..0x4020 => {}                              // unused (test mode)
            0x4020.. => self.cartridge.write(address, value), // cartridge
        }
    }
}

impl Reset for MainBus {
    fn reset(&mut self) {
        self.apu.reset();
    }
}

impl SignalProvider for MainBus {
    fn interrupt_requested(&self) -> Option<Interrupt> {
        // check for nmi first

        // then irq
        if self.apu.interrupt_status() {
            Some(Interrupt::Irq)
        } else {
            None
        }
    }

    fn dma_requested(&self) -> Option<DmaSource> {
        self.apu.dma_status().take().map(DmaSource::Apu)
    }
}
