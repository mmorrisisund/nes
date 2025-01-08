use crate::{
    apu::{Apu, ApuRegisters, Channel},
    cartridge::Cartridge,
    common::{
        mem::{MemAccess, Reset},
        interconnect::{InterruptSignaler, SignalProvider},
    },
    ppu::Ppu,
};

pub struct Bus {
    ram: [u8; 0x800],
    pub apu: Apu,
    ppu: Ppu,
    cartridge: Cartridge,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            ram: [0; 0x800],
            apu: Apu::new(),
            ppu: Ppu {},
            cartridge: Cartridge::empty(),
        }
    }

    pub fn load_cart(&mut self, cartridge: Cartridge) {
        self.cartridge = cartridge;
    }

    pub fn drain_audio_buffer(&mut self) -> Vec<f32> {
        self.apu.drain_audio_buffer()
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

impl MemAccess for Bus {
    fn peek(&self, _address: u16) -> u8 {
        todo!()
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..0x2000 => self.ram[(address & 0x7ff) as usize], // ram
            0x2000..0x4000 => {
                // TODO: just used for apu testing before ppu is done
                if address == 0x2002 {
                    0x80
                } else {
                    0
                }
            } // ppu
            0x4000..0x4014 | 0x4017 => 0,
            0x4015 => self.apu.read_status(),
            0x4014 => 0,                              // ppu
            0x4016 => 0,                              // controller
            0x4018..0x4020 => 0,                      // unused (test mode)
            0x4020.. => self.cartridge.read(address), // cartridge
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..0x2000 => self.ram[(address & 0x7ff) as usize] = data, // ram
            0x2000..0x4000 => {}                                           // ppu
            0x4000 => self.apu.write_ctrl(Channel::Pulse1, data),
            0x4001 => self.apu.write_sweep(Channel::Pulse1, data),
            0x4002 => self.apu.write_timer_lo(Channel::Pulse1, data),
            0x4003 => self.apu.write_timer_hi(Channel::Pulse1, data),
            0x4004 => self.apu.write_ctrl(Channel::Pulse2, data),
            0x4005 => self.apu.write_sweep(Channel::Pulse2, data),
            0x4006 => self.apu.write_timer_lo(Channel::Pulse2, data),
            0x4007 => self.apu.write_timer_hi(Channel::Pulse2, data),
            0x4008 => self.apu.write_ctrl(Channel::Triangle, data),
            0x4009 => {} // unused
            0x400a => self.apu.write_timer_lo(Channel::Triangle, data),
            0x400b => self.apu.write_timer_hi(Channel::Triangle, data),
            0x400c => self.apu.write_ctrl(Channel::Noise, data),
            0x400d => {} // unused
            0x400e => self.apu.write_timer_lo(Channel::Noise, data),
            0x400f => self.apu.write_length(Channel::Noise, data),
            0x4010 => self.apu.write_timer_lo(Channel::Dmc, data),
            0x4011 => self.apu.write_dmc_output(data),
            0x4012 => self.apu.write_dmc_addr(data),
            0x4013 => self.apu.write_length(Channel::Dmc, data),
            0x4015 => self.apu.write_status(data),
            0x4017 => self.apu.write_frame_counter(data),
            0x4014 => {}                                     // ppu
            0x4016 => {}                                     // controller
            0x4018..0x4020 => {}                             // unused (test mode)
            0x4020.. => self.cartridge.write(address, data), // cartridge
        }
    }
}

impl Reset for Bus {
    fn reset(&mut self) {
        self.apu.reset();
    }
}

impl SignalProvider for Bus {
    fn nmi_requested(&self) -> bool {
        self.ppu.interrupt_status()
    }

    fn clear_nmi(&mut self) {
        self.ppu.clear_interrupt();
    }

    fn interrupt_requested(&self) -> bool {
        self.apu.interrupt_status()
    }

    fn clear_interrupt(&mut self) {
        self.apu.clear_interrupt();
    }
}
