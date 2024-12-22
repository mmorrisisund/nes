use crate::{cartridge::Cartridge, common::Bus};

pub struct MainBus {
    cartridge: Cartridge,
    ram: [u8; 0x800],
}

impl Bus for MainBus {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..0x2000 => self.ram[(address & 0x7ff) as usize], // ram
            0x2000..0x4000 => 0,                                    // ppu
            0x4000..0x4018 => 0,                                    // apu
            0x4018..0x4020 => 0,                                    // unused
            0x4020.. => self.cartridge.read(address),               // cartridge
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..0x2000 => self.ram[(address & 0x7ff) as usize] = data, // ram
            0x2000..0x4000 => {}                                           // ppu
            0x4000..0x4018 => {}                                           // apu
            0x4018..0x4020 => {}                                           // unused
            0x4020.. => self.cartridge.write(address, data),               // cartridge
        }
    }
}
