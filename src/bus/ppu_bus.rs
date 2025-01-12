use crate::common::mem::MemAccess;

const VRAM_SIZE: usize = 2048;

#[derive(Debug)]
pub struct PpuBus {
    chr_rom: Vec<u8>,
    vram: [u8; VRAM_SIZE],
}

impl Default for PpuBus {
    fn default() -> Self {
        Self {
            chr_rom: Vec::new(),
            vram: [0; VRAM_SIZE],
        }
    }
}

impl MemAccess for PpuBus {
    fn read(&mut self, address: u16) -> u8 {
        todo!()
    }

    fn peek(&self, address: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, address: u16, value: u8) {
        todo!()
    }
}
