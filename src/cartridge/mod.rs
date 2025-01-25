mod error;

use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;

use crate::common::mem::{MemAccess, Memory, Reset};
pub use error::CartridgeError;

#[derive(Debug)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

#[allow(dead_code)] // TODO: Remove once ppu/mappers implemented
#[derive(Debug)]
struct INesHeader {
    prg_rom_size: u8,
    chr_rom_size: u8,
    mirroring: Mirroring,
    has_battery_ram: bool,
    has_trainer: bool,
    mapper: u8,
    prg_ram_size: u8,
}

impl INesHeader {
    pub fn from_bytes(header: &[u8]) -> Result<Self, CartridgeError> {
        if header.len() < 16 {
            return Err(CartridgeError::InvalidHeader);
        }
        Self::check_magic(&header[..4])?;

        let prg_rom_size = header[4];
        let chr_rom_size = header[5];
        let control1 = header[6];
        let control2 = header[7];
        let prg_ram_size = header[8];
        let display_type = header[9];
        let reserved = &header[10..16];

        let (mirroring, has_battery_ram, has_trainer, lo) = Self::parse_control1(control1);

        let (is_ines2, hi) = Self::parse_control2(control2);
        let mapper = hi | lo;

        if is_ines2 {
            return Err(CartridgeError::UnsupportedVersion);
        }

        if reserved.iter().any(|byte| *byte != 0) {
            return Err(CartridgeError::InvalidHeader);
        }

        if display_type != 0 {
            return Err(CartridgeError::PALNotSupported);
        }

        Ok(INesHeader {
            prg_rom_size,
            chr_rom_size,
            mirroring,
            has_battery_ram,
            has_trainer,
            mapper,
            prg_ram_size,
        })
    }

    fn empty() -> Self {
        Self::from_bytes(&[0x4E, 0x45, 0x53, 0x1A, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap()
    }

    fn check_magic(bytes: &[u8]) -> Result<(), CartridgeError> {
        let magic_bytes = [0x4E, 0x45, 0x53, 0x1A];

        if bytes == magic_bytes {
            Ok(())
        } else {
            Err(CartridgeError::InvalidHeader)
        }
    }

    fn parse_control1(mut byte: u8) -> (Mirroring, bool, bool, u8) {
        let has_vertical = byte & 0x01 == 0;
        byte >>= 1;

        let has_battery = byte & 0x01 == 1;
        byte >>= 1;

        let has_trainer = byte & 0x01 == 1;
        byte >>= 1;

        let has_four_screen = byte & 0x01 == 1;
        byte >>= 1;

        let lo = byte & 0xf;

        let mirroring = match (has_vertical, has_four_screen) {
            (_, true) => Mirroring::FourScreen,
            (true, _) => Mirroring::Vertical,
            (false, _) => Mirroring::Horizontal,
        };

        (mirroring, has_battery, has_trainer, lo)
    }

    fn parse_control2(mut byte: u8) -> (bool, u8) {
        // ignore first 2 bits, not in official spec
        byte >>= 2;

        let is_ines2 = byte & 0b10 == 2;
        byte >>= 2;

        let hi = byte & 0xf;

        (is_ines2, hi << 4)
    }
}

#[allow(dead_code)] // TODO: Remove once ppu/mappers implemented
pub struct Cartridge {
    header: INesHeader,
    prg_rom: Vec<u8>,
    pub chr_rom: Memory,
    prg_ram: Vec<u8>,
}

impl Cartridge {
    pub fn from_file<P: AsRef<Path>>(file_path: P) -> Result<Self, CartridgeError> {
        if let Some(extension) = file_path.as_ref().extension() {
            if extension == "nes" {
                let mut file = File::open(file_path)?;

                let mut header = [0; 16];
                file.read_exact(&mut header)?;
                let header = INesHeader::from_bytes(&header)?;

                if header.has_trainer {
                    let mut _trainer = [0; 512];
                    file.read_exact(&mut _trainer)?;
                }

                let mut prg_rom = vec![0; header.prg_rom_size as usize * 16 * 1024];
                file.read_exact(&mut prg_rom)?;

                let mut chr_rom = Memory::from(vec![0; header.chr_rom_size as usize * 8 * 1024]);
                file.read_exact(&mut chr_rom)?;

                // TODO: load sram (saved data; batter pack)
                let ram_size = std::cmp::max(1, header.prg_ram_size);
                let prg_ram = vec![0; ram_size as usize * 8 * 1024];

                let current_position = file.stream_position()?;
                let end = file.seek(std::io::SeekFrom::End(0))?;
                if current_position != end {
                    return Err(CartridgeError::FileTooLarge(end - current_position));
                }

                Ok(Self {
                    header,
                    prg_rom,
                    chr_rom,
                    prg_ram,
                })
            } else {
                Err(CartridgeError::UnsupportedFile)
            }
        } else {
            Err(CartridgeError::UnsupportedFile)
        }
    }

    pub fn empty() -> Self {
        Self {
            header: INesHeader::empty(),
            prg_rom: Vec::new(),
            chr_rom: Memory::new(),
            prg_ram: Vec::new(),
        }
    }

    // TODO: Maybe make it so cartridges can be created from Vec<u8>
    #[cfg(test)]
    pub fn set_prg_rom(&mut self, new_rom: Vec<u8>) {
        self.prg_rom = new_rom;
    }

    #[cfg(test)]
    pub fn get_prg_rom(&self) -> Vec<u8> {
        self.prg_rom.clone()
    }

    pub fn dump_chr_rom(&self) -> &Vec<u8> {
        &self.chr_rom
    }
}

impl Reset for Cartridge {
    fn reset(&mut self) {
        todo!()
    }
}

impl MemAccess for Cartridge {
    fn read(&mut self, address: u16) -> u8 {
        self.peek(address)
    }

    fn peek(&self, address: u16) -> u8 {
        match address {
            0x0000..0x2000 => todo!(), // ppu
            0x2000..0x6000 => unreachable!(),
            0x6000..0x8000 => self.prg_ram[address as usize - 0x6000],

            0x8000.. => {
                // TODO: eventually to be handled by mapper
                let mut addr = address as usize - 0x8000;

                if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
                    addr &= 0x3fff;
                }
                self.prg_rom[addr]
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..0x2000 => todo!(), // ppu
            0x2000..0x6000 => unreachable!(),
            0x6000..0x8000 => self.prg_ram[address as usize - 0x6000] = data,
            0x8000.. => {
                // TODO: eventually to be handled by mapper
                let mut addr = address as usize - 0x8000;

                if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
                    addr &= 0x3fff;
                }
                self.prg_rom[addr] = data;
            }
        }
    }
}

impl Debug for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cartridge")
            .field("header", &self.header)
            .field("prg_rom", &self.prg_rom.len())
            .field("chr_rom", &self.chr_rom)
            .field("prg_ram", &self.prg_ram.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    const HEADER: [u8; 16] = [
        0x4e, 0x45, 0x53, 0x1a, 0x10, 0x10, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ];

    #[test]
    fn check_magic_success() {
        let header = INesHeader::from_bytes(&HEADER);

        assert!(header.is_ok());
    }

    #[test]
    fn check_magic_failure() {
        let header = INesHeader::from_bytes(&[0x01, 0x02, 0x03, 0x04]);

        assert!(header.is_err());
    }

    #[test]
    fn file_not_found() {
        let err = Cartridge::from_file("../fake_dir/not_real.nes")
            .err()
            .expect("should be an error");

        if let CartridgeError::IOError(file_err) = err {
            assert_eq!(file_err.kind(), ErrorKind::NotFound);
        } else {
            panic!("did not receive IO Error, received: {err:?}");
        }
    }

    #[test]
    fn invalid_extension() {
        let err = Cartridge::from_file("../fake_dir/game.notnes")
            .err()
            .expect("should be an error");

        if let CartridgeError::UnsupportedFile = err {
            // success
        } else {
            panic!("did not receive UnsupportedFile error, received: {err:?}");
        }
    }

    #[test]
    fn invalid_header() {
        // let f = include_bytes!("../test roms/cartridge_tests/wrong_header.nes");
        let err = Cartridge::from_file("./src/test_roms/cartridge/wrong_header.nes")
            .err()
            .expect("should be an error");

        if let CartridgeError::InvalidHeader = err {
            // success
        } else {
            panic!("did not receive InvalidHeader error, received: {err:?}");
        }
    }

    #[test]
    fn large_file() {
        let err = Cartridge::from_file("./src/test_roms/cartridge/large_file.nes")
            .err()
            .expect("should be an error");

        if let CartridgeError::FileTooLarge(_) = err {
            // success
        } else {
            panic!("did not receive a FileTooLarge error, received: {err:?}");
        }
    }

    #[test]
    fn valid_cartridge_created() {
        let cart = Cartridge::from_file("./src/test_roms/cartridge/creation.nes")
            .expect("should be successful");

        for c in cart.prg_rom {
            assert_eq!(c, 0xff);
        }

        for c in cart.chr_rom {
            assert_eq!(c, 0xee);
        }
    }
}
