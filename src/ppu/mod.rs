mod registers;

use registers::{
    address::AddressRegister, ctrl::ControlRegister, mask::MaskRegister, status::StatusRegister,
};

use crate::common::interconnect::InterruptSignaler;

#[derive(Debug, Default)]
pub struct Ppu {
    control: ControlRegister,
    mask: MaskRegister,
    status: StatusRegister,
    v_reg: AddressRegister, // internal v register
    t_reg: AddressRegister, // internal t register

    write_first: bool, // internal w register
}

impl Ppu {
    // Register 0x2000 (W)
    pub fn write_control(&mut self, value: u8) {
        self.control.write(value);
        self.t_reg.set_nametable_select(value);
    }

    // Register 0x2001 (W)
    pub fn write_mask(&mut self, value: u8) {
        self.mask.set_byte(value);
    }

    // Register 0x2002 (R)
    pub fn read_status(&mut self) -> u8 {
        self.write_first = true;
        self.peek_status()
    }

    // (not mutating)
    pub fn peek_status(&self) -> u8 {
        self.status.value()
    }

    // Register 0x2003 (RW)
    pub fn read_oam_addr(&self) -> u8 {
        todo!()
    }

    pub fn write_oam_addr(&mut self, value: u8) {
        todo!()
    }

    // Register 0x2004 (RW)
    pub fn read_oam_data(&self) -> u8 {
        todo!()
    }

    pub fn write_oam_data(&mut self, value: u8) {
        todo!()
    }

    // Register 0x2005 (W)
    pub fn write_scroll(&mut self, value: u8) {
        todo!()
    }

    // Register 0x2006 (W)
    pub fn write_ppu_addr(&mut self, value: u8) {
        todo!()
    }

    // Register 0x2007 (RW)
    pub fn read_ppu_data(&mut self) -> u8 {
        todo!()
    }

    // non mutating
    pub fn peek_ppu_data(&self) -> u8 {
        todo!()
    }

    pub fn write_ppu_data(&mut self, value: u8) {
        todo!()
    }
}

impl InterruptSignaler for Ppu {
    fn interrupt_status(&self) -> bool {
        false
    }
}
