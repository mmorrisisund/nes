use crate::{
    cartridge::Mirroring,
    common::mem::{MemAccess, Memory},
};

const VRAM_SIZE: usize = 2048;
const PALETTE_SIZE: usize = 32;

#[derive(Debug)]
pub struct PpuBus {
    pub chr_rom: Memory,
    vram: [u8; VRAM_SIZE],
    mirroring: Mirroring,
    palette: [u8; PALETTE_SIZE],
}

impl Default for PpuBus {
    fn default() -> Self {
        Self::new(Mirroring::Horizontal)
    }
}

impl PpuBus {
    pub fn new(mirroring: Mirroring) -> Self {
        Self {
            chr_rom: Memory::new(),
            vram: [0x00; VRAM_SIZE],
            mirroring,
            palette: [0; PALETTE_SIZE],
        }
    }

    pub fn load_chr_rom(&mut self, chr_rom: Memory) {
        self.chr_rom = chr_rom;
    }
}

impl PpuBus {
    fn get_vram_address(&self, address: u16) -> usize {
        // TODO: all of this is wrong, impacts actual scrolling
        let vram_index = address & 0xfff;
        // let nametable = vram_index / 0x400;
        // let mirrored_address = match self.mirroring {
        //     Mirroring::Horizontal if matches!(nametable, 1 | 2) => vram_index - 0x400,
        //     _ => vram_index,
        // };
        // (mirrored_address & 0x7ff) as usize
        ((vram_index & 0x3ff) & 0x7ff) as usize
    }
}

impl MemAccess for PpuBus {
    fn read(&mut self, address: u16) -> u8 {
        self.peek(address)
    }

    fn peek(&self, address: u16) -> u8 {
        match address {
            0x0000..0x2000 => self.chr_rom[address as usize],
            0x2000..0x3f00 => self.vram[self.get_vram_address(address)],
            0x3f00..0x4000 => self.palette[(address & 0x1f) as usize],
            _ => self.peek(address & 0x3fff),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..0x2000 => {}
            0x2000..0x3f00 => self.vram[self.get_vram_address(address)] = value,
            0x3f00..0x4000 => self.palette[(address & 0x1f) as usize] = value,
            _ => self.write(address & 0x3fff, value),
        }
    }
}
