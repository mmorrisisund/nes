mod registers;

use registers::{
    address::AddressRegister, ctrl::ControlRegister, mask::MaskRegister, status::StatusRegister,
};

use crate::{
    bus::PpuBus,
    common::{interconnect::InterruptSignaler, mem::MemAccess},
};

#[derive(Debug)]
pub struct Ppu {
    bus: PpuBus,
    control: ControlRegister,
    mask: MaskRegister,
    status: StatusRegister,
    v_addr: AddressRegister, // internal v register
    t_addr: AddressRegister, // internal t register
    first_write: bool,       // internal w register
    fine_x: u8,              // internal x register
    oam_addr: u8,
    oam_data: [u8; 256],
    vram_buffer: u8,
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            oam_data: [0; 256],
            ..Default::default()
        }
    }

    fn vram_increment_addr(&mut self) {
        // TODO: Implement alternate behavior when rendering
        self.v_addr.increment_addr(self.control.vram_increment());
    }
}

// register access
impl Ppu {
    // Register 0x2000 (W)
    pub fn write_control(&mut self, value: u8) {
        self.control.write(value);
        self.t_addr.set_nametable_select(value);
    }

    // Register 0x2001 (W)
    pub fn write_mask(&mut self, value: u8) {
        self.mask.set_byte(value);
    }

    // Register 0x2002 (R)
    pub fn read_status(&mut self) -> u8 {
        self.first_write = true;
        self.peek_status()
    }

    // (not mutating)
    pub fn peek_status(&self) -> u8 {
        self.status.value()
    }

    // Register 0x2003 (W)

    pub fn write_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }

    // Register 0x2004 (RW)
    pub fn read_oam_data(&self) -> u8 {
        // TODO: possibly return value from secondary oam during rendering and sprite eval
        self.oam_data[self.oam_addr as usize]
    }

    pub fn write_oam_data(&mut self, value: u8) {
        // TODO: implement glitch on write during render
        // TODO: should only write when not rendering
        self.oam_data[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    // Register 0x2005 (W)
    pub fn write_scroll(&mut self, value: u8) {
        if self.first_write {
            self.t_addr.set_coarse_x(value >> 3);
            self.fine_x = value & 0x07;
        } else {
            self.t_addr.set_coarse_y(value >> 3);
            self.t_addr.set_fine_y(value & 0x07);
        }
        self.first_write = !self.first_write;
    }

    // Register 0x2006 (W)
    pub fn write_ppu_addr(&mut self, value: u8) {
        if self.first_write {
            self.t_addr.set_high(value);
        } else {
            self.t_addr.set_low(value);
            self.v_addr = self.t_addr;
        }
        self.first_write = !self.first_write;
    }

    // Register 0x2007 (RW)
    pub fn read_ppu_data(&mut self) -> u8 {
        let address = self.v_addr.get_addr();
        let buffered_value = self.vram_buffer;
        self.vram_buffer = self.bus.read(address);
        self.vram_increment_addr();

        if matches!(address & 0x3fff, 0x3f00..0x4000) {
            self.vram_buffer
        } else {
            buffered_value
        }
    }

    // non mutating
    pub fn peek_ppu_data(&self) -> u8 {
        let address = self.v_addr.get_addr();
        let value = self.bus.peek(address);

        if matches!(address & 0x3fff, 0x3f00..0x4000) {
            value
        } else {
            self.vram_buffer
        }
    }

    pub fn write_ppu_data(&mut self, value: u8) {
        let address = self.v_addr.get_addr();
        self.bus.write(address, value);
        self.vram_increment_addr();
    }
}

impl InterruptSignaler for Ppu {
    fn interrupt_status(&self) -> bool {
        false
    }
}
