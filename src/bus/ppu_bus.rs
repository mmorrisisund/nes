use crate::{cartridge::Mirroring, common::mem::MemAccess};

const VRAM_SIZE: usize = 2048;
const PALETTE_SIZE: usize = 32;

#[derive(Debug)]
pub struct PpuBus {
    chr_rom: Vec<u8>,
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
            chr_rom: Vec::new(),
            vram: [0; VRAM_SIZE],
            mirroring,
            palette: [0; PALETTE_SIZE],
        }
    }
}

impl PpuBus {
    // Horizontal:
    //   [ A ] [ a ]
    //   [ B ] [ b ]

    // Vertical:
    //   [ A ] [ B ]
    //   [ a ] [ b ]
    fn get_vram_address(&self, address: u16) -> usize {
        let vram_index = address & 0xfff;
        let nametable = vram_index / 0x400;
        let mirrored_address = match self.mirroring {
            Mirroring::Horizontal if matches!(nametable, 1 | 2) => vram_index - 0x400,
            _ => vram_index,
        };

        (mirrored_address & 0x7ff) as usize
    }
}

impl MemAccess for PpuBus {
    fn read(&mut self, address: u16) -> u8 {
        self.peek(address)
    }

    fn peek(&self, address: u16) -> u8 {
        match address {
            0x0000..0x2000 => self.chr_rom.get(address as usize).copied().unwrap_or(0),
            0x2000..0x3f00 => self.vram[self.get_vram_address(address)],
            0x3f00..0x4000 => self.palette[(address & 0x1f) as usize],
            _ => self.peek(address & 0x3fff),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..0x2000 => {}
            0x2000..0x3f00 => self.vram[self.get_vram_address(address)] = value,
            0x3f00..0x4000 => {}
            _ => self.write(address & 0x3fff, value),
        }
    }
}
