mod frame;
mod registers;
mod sprite;
mod tile;

use std::cell::Cell;

use frame::Frame;
use registers::{
    address::AddressRegister, ctrl::ControlRegister, mask::MaskRegister, status::StatusRegister,
};
use sprite::Sprite;
use tile::Tile;

use crate::{
    bus::PpuBus,
    common::{
        interconnect::{DmaSource, Interrupt, SignalProvider},
        mem::MemAccess,
    },
};

const WIDTH: u16 = 256;
const HEIGHT: u16 = 240;
const TOTAL_PIXELS: usize = (WIDTH * HEIGHT) as usize;
const PALETTE_START: u16 = 0x3f00;
const SPRITE_PALETTE: u16 = 0x0010;
const BG_PALETTE: u16 = 0x0000;

const SECONDARY_OAM_SIZE: usize = 32;
const MAX_SPRITES: usize = 8;

// cycles
const VBLANK_START_CYCLE: u16 = 1;
const VISIBLE_START: u16 = 1; // 2 cycles each for 4 fetches, 32 tiles
const VISIBLE_END: u16 = 256;
const OAM_CLEAR_START: u16 = 1;
const OAM_CLEAR_END: u16 = 64;
const SPRITE_EVAL_START: u16 = 65;
const SPRITE_EVAL_END: u16 = 256;
const INC_Y: u16 = 256;
const COPY_X: u16 = 257;
const SPRITE_FETCH_START: u16 = 257; // 2 cycles each for 4 fetches, 8 tiles
const SPRITE_FETCH_END: u16 = 320;
const COPY_Y_START: u16 = 280;
const COPY_Y_END: u16 = 304;
const BG_PREFETCH_START: u16 = 321; // 2 cycles each for 4 fetches, 2 tiles
const BG_PREFETCH_END: u16 = 336;
const DUMMY_READ_START: u16 = 337;
const ODD_SKIP_CYCLE: u16 = 339; // odd frames, line 261, skip last cycle
const CYCLE_END: u16 = 340;

// scanlines
const VISIBLE_SCANLINE_END: u16 = 239;
const VBLANK_START_LINE: u16 = 241;
const PRERENDER_LINE: u16 = 261;

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
    oam_fetch_buffer: u8,
    vram_buffer: u8,

    secondary_oam_addr: u8,
    secondary_oam_data: [u8; SECONDARY_OAM_SIZE],
    sprites: [Sprite; MAX_SPRITES],

    // State used during sprite evaluation
    sprite_in_range: bool,
    sprite_n: u8,
    byte_n: u8,
    eval_done: bool,
    sprite_count: u16,

    current_tile: Tile,
    frame: Frame,

    scanline: u16,
    cycle: u16,
    nmi: Cell<bool>,
    prevent_vbl: bool,

    warmup: bool,
    total_cycles: u64,
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            bus: PpuBus::default(),
            control: ControlRegister::default(),
            mask: MaskRegister::default(),
            status: StatusRegister::default(),
            v_addr: AddressRegister::default(), // internal v register
            t_addr: AddressRegister::default(), // internal t register
            first_write: true,                  // internal w register
            fine_x: 0,                          // internal x register
            oam_addr: 0,
            oam_data: [0; 256],
            oam_fetch_buffer: 0,

            secondary_oam_addr: 0,
            secondary_oam_data: [0; 32],
            sprites: [Sprite::default(); MAX_SPRITES],

            sprite_in_range: false,
            sprite_n: 0,
            byte_n: 0,
            eval_done: false,
            sprite_count: 0,

            current_tile: Tile::default(),
            frame: Frame::new(),

            vram_buffer: 0,

            scanline: 0,
            cycle: 0, // 27 mismatch between cpu and ppu at startup
            nmi: Cell::new(false),
            prevent_vbl: false,

            warmup: true,
            total_cycles: 0,
        }
    }

    pub fn clock(&mut self) {
        self.total_cycles += 1;

        if self.cycle >= CYCLE_END {
            self.cycle = 0;
            self.scanline += 1;

            if self.scanline > PRERENDER_LINE {
                self.scanline = 0;
            }
        } else {
            self.cycle += 1;
            self.tick();

            if self.cycle == VBLANK_START_CYCLE {
                if self.scanline == VBLANK_START_LINE {
                    self.start_vblank();
                } else if self.scanline == PRERENDER_LINE {
                    self.end_vblank();
                }
            }
        }
    }

    fn tick(&mut self) {
        let is_visible_cycle = matches!(self.cycle, VISIBLE_START..=VISIBLE_END);
        let is_bg_prefetch_cycle = matches!(self.cycle, BG_PREFETCH_START..=BG_PREFETCH_END);
        let is_bg_fetch_cycle = is_visible_cycle || is_bg_prefetch_cycle;
        let is_sprite_fetch_cycle = matches!(self.cycle, SPRITE_FETCH_START..=SPRITE_FETCH_END);
        let is_dummy_bg_cycle = matches!(self.cycle, DUMMY_READ_START..=CYCLE_END);
        let is_dummy_sprite_cycle = matches!(self.cycle, BG_PREFETCH_START..=CYCLE_END);

        let is_visible_scanline = self.scanline <= VISIBLE_SCANLINE_END;
        let is_prerender_scanline = self.scanline == PRERENDER_LINE;
        let is_render_scanline = is_visible_scanline || is_prerender_scanline;

        if self.mask.rendering_enabled() {
            if is_visible_scanline {
                match self.cycle {
                    OAM_CLEAR_START..=OAM_CLEAR_END => {
                        self.oam_fetch_buffer = 0xff;
                        self.secondary_oam_data.fill(0xff);
                    }
                    SPRITE_EVAL_START..=SPRITE_EVAL_END => self.evaluate_sprites(),
                    SPRITE_FETCH_START..=SPRITE_FETCH_END => self.oam_addr = 0,
                    _ => {}
                }
            }

            if is_render_scanline {
                if is_bg_fetch_cycle {
                    self.fetch_bg();

                    if self.cycle & 0x07 == 0x00 {
                        self.v_addr.increment_x();
                        self.current_tile.load_next_tile();
                    }
                } else if is_dummy_bg_cycle {
                    self.fetch_nt_byte();
                }

                if is_sprite_fetch_cycle {
                    // possibly clear sprite hit tracker
                    self.fetch_sprite();
                } else if is_dummy_sprite_cycle {
                    self.oam_fetch_buffer = self.secondary_oam_data[0];
                }

                if is_prerender_scanline {
                    // https://www.nesdev.org/wiki/PPU_registers#OAMADDR_precautions
                    // if OAMADDR is not less than eight when rendering starts, the eight bytes starting at OAMADDR & 0xF8 are copied to the first eight bytes of OAM
                    match self.cycle {
                        VISIBLE_START..=8 if self.oam_addr >= 0x08 => {
                            let offset = self.cycle as usize - 1;
                            let index = (self.oam_addr as usize & 0xf8) + offset;
                            self.oam_data[offset] = self.oam_data[index];
                        }
                        COPY_Y_START..=COPY_Y_END => {
                            self.v_addr
                                .set_vertical_bits(self.t_addr.get_vertical_bits());
                        }
                        _ => {}
                    }
                }

                match self.cycle {
                    INC_Y => self.v_addr.increment_y(),
                    COPY_X => {
                        self.v_addr
                            .set_horizontal_bits(self.t_addr.get_horizontal_bits());
                    }
                    _ => (),
                }

                if self.cycle == ODD_SKIP_CYCLE {
                    self.cycle = CYCLE_END;
                }
            }
        }

        // render pixel, always done. even if rendering disaled
        if is_visible_scanline && is_visible_cycle {
            self.render_pixel();
        }

        // updated shift registers
        self.current_tile.shift();
    }

    fn render_pixel(&mut self) {
        let x = self.cycle - 1;
        let y = self.scanline;
        let address = self.v_addr.address();

        // When the PPU isn't rendering, its v register specifies the current VRAM address
        // (and is output on the PPU's address pins). Whenever the low 14 bits of v point into
        // palette RAM ($3F00-$3FFF), the PPU will continuously draw the color at that address
        // instead of the EXT input, overriding the backdrop color.
        let color = if (address & PALETTE_START) == PALETTE_START {
            self.bus.read(address)
        } else if self.mask.rendering_enabled() {
            self.bus.read(self.color_address())
        } else {
            self.bus.read(PALETTE_START)
        };

        self.frame.set_pixel(x, y, color);
    }

    fn color_address(&self) -> u16 {
        let x = self.cycle - 1;
        let bg_enabled = self.mask.show_background();
        let bg_clip_set = !self.mask.show_left_bg();
        let sprite_enabled = self.mask.show_sprite();
        let sprite_clip_set = !self.mask.show_left_sprite();

        // check for left 8px clip
        let bg_hide_left = x < 8 && bg_clip_set;
        let bg_color = if bg_enabled & !bg_hide_left {
            self.current_tile.pixel_color(self.fine_x) as u16
        } else {
            0
        };

        let sprite_hide_left = x < 8 && sprite_clip_set;
        if sprite_enabled && !sprite_hide_left {
            for sprite in self.sprites.iter().take(self.sprite_count.into()) {
                if x < sprite.x {
                    continue;
                }

                let x_offset = x.saturating_sub(sprite.x) as u8;
                let sprite_color = sprite.pixel_color(x_offset) as u16;

                // TODO: sprite zero hit check

                if bg_color == 0 || sprite.has_priority {
                    if sprite_color == 0 {
                        return PALETTE_START;
                    } else {
                        return PALETTE_START | SPRITE_PALETTE | sprite_color;
                    }
                }
            }
        };

        PALETTE_START | BG_PALETTE | bg_color
    }

    // https://www.nesdev.org/wiki/PPU_sprite_evaluation#Details
    fn evaluate_sprites(&mut self) {
        if self.cycle == SPRITE_EVAL_START {
            self.sprite_in_range = false;
            self.secondary_oam_addr = 0;
            self.eval_done = false;
            self.sprite_n = (self.oam_addr >> 2) & 0x3f;
            self.byte_n = self.oam_addr & 0b11;
        } else if self.cycle == SPRITE_EVAL_END {
            // sprite 0 visible thing
            self.sprite_count = (self.secondary_oam_addr >> 2) as u16; // divide by 4, 4 bytes/sprite
        }

        if self.cycle & 0x01 == 0x01 {
            // On odd cycles, data is read from (primary) OAM
            self.oam_fetch_buffer = self.oam_data[self.oam_addr as usize];
        } else {
            // On even cycles, data is written to secondary OAM (unless secondary OAM is full, in which case it will read the value in secondary OAM instead)
            let index = self.secondary_oam_addr as usize & 0x1f;
            let sprite_y = self.oam_fetch_buffer as u16;

            if self.eval_done {
                self.sprite_n = (self.sprite_n + 1) & 0x1f;
                if self.secondary_oam_addr > SECONDARY_OAM_SIZE as u8 {
                    self.oam_fetch_buffer = self.secondary_oam_data[index];
                }
            } else {
                self.sprite_in_range |=
                    !self.sprite_in_range && (sprite_y..sprite_y + 8).contains(&self.scanline);

                if self.secondary_oam_addr < SECONDARY_OAM_SIZE as u8 {
                    self.secondary_oam_data[index] = self.oam_fetch_buffer;
                    // If Y-coordinate is in range, copy remaining bytes of sprite data (OAM[n][1] thru OAM[n][3]) into secondary OAM.
                    if self.sprite_in_range {
                        self.byte_n += 1;
                        self.secondary_oam_addr += 1;

                        // sprite zero check??
                        if self.byte_n == 0x03 {
                            self.sprite_in_range = false;
                            self.byte_n = 0;
                            self.sprite_n = (self.sprite_n + 1) & 0x3f;
                            self.eval_done |= self.sprite_n == 0x00;
                        }
                    } else {
                        self.sprite_n = (self.sprite_n + 1) & 0x3f;
                        self.eval_done |= self.sprite_n == 0x00;
                    }
                } else {
                    // read from secondary oam
                    self.oam_fetch_buffer = self.secondary_oam_data[index];

                    if self.sprite_in_range {
                        self.status.set_sprite_overflow();

                        self.byte_n += 1;
                        if self.byte_n == 0x04 {
                            self.byte_n = 0;
                            self.sprite_n = (self.sprite_n + 1) & 0x3f;
                            self.eval_done = true;
                        }
                    } else {
                        self.sprite_n = (self.sprite_n + 1) & 0x3f;
                        self.byte_n = (self.byte_n + 1) & 0x03;
                        self.eval_done |= self.sprite_n == 0x00;
                    }
                }
            }

            self.oam_addr = (self.sprite_n << 2) | (self.byte_n & 0x03);
        }
    }

    /// Fetch 4 tiles and write out shift registers every 8th cycle.
    /// Each tile fetch takes 2 cycles.
    fn fetch_bg(&mut self) {
        match self.cycle & 0x07 {
            1 => self.fetch_nt_byte(),
            3 => self.fetch_attr_byte(),
            5 => {
                self.current_tile.next_pattern.low =
                    self.bus.read(self.current_tile.pattern_address)
            }
            7 => {
                self.current_tile.next_pattern.high =
                    self.bus.read(self.current_tile.pattern_address + 8)
            }
            _ => {}
        }
    }

    fn fetch_sprite(&mut self) {
        match self.cycle & 0x07 {
            1 => self.fetch_nt_byte(), // garbage read
            3 => self.fetch_nt_byte(), // garbage read.  why does this read nt, and not attr
            4 => self.load_sprites(),
            _ => {}
        }
    }

    fn fetch_nt_byte(&mut self) {
        let nt_addr = self.v_addr.nametable_addr();
        let pattern_table_id = self.bus.read(nt_addr) as u16;

        self.current_tile.pattern_address = self.control.bg_pattern_addr()
            | (pattern_table_id << 4)
            | self.v_addr.get_fine_y() as u16;
    }

    fn fetch_attr_byte(&mut self) {
        let attr_addr = self.v_addr.attribute_addr();
        let palette_selector = self.v_addr.attribute_shift(); // determine quadrant in metatile
        self.current_tile.next_palette = (self.bus.read(attr_addr) >> palette_selector) & 0b11;
    }

    fn load_sprites(&mut self) {
        // - 257-320 | 64 cycles
        // * track "fetch" index
        // * oam index = 4 * fetch index
        // * read sprite data from secondary_oam with oam index
        // * build sprite
        let sprite_index = (self.cycle - SPRITE_FETCH_START) / 8;
        let oam_index = (sprite_index << 2) as usize;

        // 0HNNNN NNNNPyyy
        // |||||| |||||+++- T: Fine Y offset, the row number within a tile
        // |||||| ||||+---- P: Bit plane (0: less significant bit; 1: more significant bit)
        // ||++++-++++----- N: Tile index from name table
        // |+-------------- H: Half of pattern table (0: "left"; 1: "right")
        // +--------------- 0: Pattern table is at $0000-$1FFF
        if let [y, tile_idx, attr, x] = self.secondary_oam_data[oam_index..oam_index + 4] {
            let y = y as u16;
            let flip_vertical = (attr & 0x80) == 0x80;
            let mut tile_idx = tile_idx as u16;
            let height = self.control.sprite_size();

            let mut fine_y = if (y..y + height).contains(&self.scanline) {
                self.scanline - y
            } else {
                0
            };

            if flip_vertical {
                fine_y = height - fine_y - 1;
            }

            // no more actual sprites left to render
            if sprite_index >= self.sprite_count {
                fine_y = 0;
                tile_idx = 0xff;
            }

            let tile_addr = if self.control.sprite_size() == 8 {
                let bank = self.control.sprite_pattern_addr();
                bank | (tile_idx << 4) | fine_y
            } else {
                let bank = (tile_idx & 0x01) * 0x1000;
                bank | ((tile_idx & 0xfe) << 4) | fine_y
            };

            if sprite_index < self.sprite_count {
                let sprite = &mut self.sprites[sprite_index as usize];
                sprite.x = x as u16;
                sprite.y = y;
                sprite.tile_lo = self.bus.read(tile_addr);
                sprite.tile_hi = self.bus.read(tile_addr + 8);
                sprite.palette = attr & 0b11;
                sprite.has_priority = (attr & 0x20) == 0;
                sprite.flip_horizontal = (attr & 0x40) == 0x40;
                sprite.flip_vertical = flip_vertical;

                // TODO: track if sprite is present for zero hit detection
            } else {
                let _ = self.bus.read(tile_addr);
                let _ = self.bus.read(tile_addr + 8);
            }
        }
    }

    fn start_vblank(&mut self) {
        // println!(
        //     "start @ {}({}): {}",
        //     self.total_cycles,
        //     self.total_cycles / 3,
        //     self.warmup
        // );
        if !self.prevent_vbl {
            self.status.set_vblank_started();
            if self.control.nmi_enable() {
                self.nmi.set(true);
            }
        }
        self.prevent_vbl = false;
    }

    fn end_vblank(&mut self) {
        // println!(
        //     "end @ {}({}): {}",
        //     self.total_cycles,
        //     self.total_cycles / 3,
        //     self.warmup
        // );
        self.warmup = false;
        self.status.clear();
        self.nmi.set(false);
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
        if self.warmup {
            return;
        }

        let prior_enable = self.control.nmi_enable();
        self.control.write(value);
        self.t_addr.set_nametable_select(value);

        if !self.control.nmi_enable() {
            self.nmi.set(false);
        } else if self.status.is_in_vblank() && !prior_enable {
            self.nmi.set(true);
        }
    }

    // Register 0x2001 (W)
    pub fn write_mask(&mut self, value: u8) {
        if self.warmup {
            return;
        }
        self.mask.set_byte(value);
    }

    // Register 0x2002 (R)
    pub fn read_status(&mut self) -> u8 {
        let status = self.peek_status();

        self.first_write = true; // `reset `latch
        self.status.clear_vblank_started(); // reset vblank flag

        if self.scanline == VBLANK_START_LINE && self.cycle == VBLANK_START_CYCLE - 1 {
            // https://www.nesdev.org/wiki/PPU_frame_timing#VBL_Flag_Timing
            self.prevent_vbl = true;
        }
        status
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
        if self.warmup {
            return;
        }
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
        if self.warmup {
            return;
        }
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
        let address = self.v_addr.address();
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
        let address = self.v_addr.address();
        let value = self.bus.peek(address);

        if matches!(address & 0x3fff, 0x3f00..0x4000) {
            value
        } else {
            self.vram_buffer
        }
    }

    pub fn write_ppu_data(&mut self, value: u8) {
        let address = self.v_addr.address();
        self.bus.write(address, value);
        self.vram_increment_addr();
    }
}

impl SignalProvider for Ppu {
    fn interrupt_requested(&self) -> Option<Interrupt> {
        if self.nmi.get() {
            self.nmi.set(false);
            Some(Interrupt::Nmi)
        } else {
            None
        }
    }

    fn dma_requested(&self) -> Option<DmaSource> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::blargg_test;

    const TEST_ROOT: &str = "./src/test_roms/ppu";

    mod vbl_nmi {
        use super::*;

        const TEST_FOLDER: &str = "/vbl_nmi/rom_singles";

        blargg_test!(vbl_basics, TEST_ROOT, TEST_FOLDER, "/01-vbl_basics.nes");
        blargg_test!(vbl_set_time, TEST_ROOT, TEST_FOLDER, "/02-vbl_set_time.nes");
        blargg_test!(
            vbl_clear_time,
            TEST_ROOT,
            TEST_FOLDER,
            "/03-vbl_clear_time.nes"
        );
        blargg_test!(nmi_control, TEST_ROOT, TEST_FOLDER, "/04-nmi_control.nes");
        blargg_test!(nmi_timing, TEST_ROOT, TEST_FOLDER, "/05-nmi_timing.nes");
        // blargg_test!(suppression, TEST_ROOT, TEST_FOLDER, "/06-suppression.nes");
    }
}
