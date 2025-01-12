use crate::{cartridge::Mirroring, common::mem::MemAccess};

const VRAM_SIZE: usize = 2048;

#[derive(Debug)]
pub struct PpuBus {
    chr_rom: Vec<u8>,
    vram: [u8; VRAM_SIZE],
    mirroring: Mirroring,
}

impl Default for PpuBus {
    fn default() -> Self {
        Self {
            chr_rom: Vec::new(),
            vram: [0; VRAM_SIZE],
            mirroring: Mirroring::Horizontal,
        }
    }
}

impl PpuBus {
    pub fn new(mirroring: Mirroring) -> Self {
        Self {
            chr_rom: Vec::new(),
            vram: [0; VRAM_SIZE],
            mirroring,
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
            0x0000..0x2000 => todo!("peek chr_rom"),
            0x2000..0x3f00 => self.vram[self.get_vram_address(address)],
            0x3f00..0x4000 => todo!("peek palette"),
            _ => self.peek(address & 0x3fff),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..0x2000 => todo!("write chr_rom"),
            0x2000..0x3f00 => self.vram[self.get_vram_address(address)] = value,
            0x3f00..0x4000 => todo!("write palette"),
            _ => self.write(address & 0x3fff, value),
        }
    }
}
