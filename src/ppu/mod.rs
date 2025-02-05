mod frame;
mod registers;
mod sprite;
mod tile;

use std::cell::Cell;

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::{
    bus::PpuBus,
    common::{interconnect::InterruptSignaler, mem::MemAccess},
};
use frame::Frame;
use registers::{
    address::{self, AddressRegister},
    ctrl::ControlRegister,
    mask::MaskRegister,
    status::StatusRegister,
};
use sprite::Sprite;
use tile::Tile;

pub const WIDTH: u32 = 256;
pub const HEIGHT: u32 = 240;
pub const TOTAL_PIXELS: usize = (WIDTH * HEIGHT) as usize;
const PALETTE_START: u16 = 0x3f00;
const PALETTE_END: u16 = 0x3f20;

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
const ODD_SKIP_CYCLE: u16 = 339; // odd frames, line 261, skip last cycle
const CYCLE_END: u16 = 340;

// scanlines
const VISIBLE_SCANLINE_START: u16 = 0;
const VISIBLE_SCANLINE_END: u16 = 239;
const VBLANK_START_LINE: u16 = 241;
const PRERENDER_LINE: u16 = 261;

#[derive(Debug, Serialize, Deserialize)]
pub struct Ppu {
    pub bus: PpuBus,
    control: ControlRegister,
    mask: MaskRegister,
    status: StatusRegister,
    v_addr: AddressRegister, // internal v register
    t_addr: AddressRegister, // internal t register
    first_write: bool,       // internal w register
    fine_x: u8,              // internal x register
    oam_idx: u8,
    #[serde(with = "BigArray")]
    oam_data: [u8; 256],
    oam_buffer: u8,
    vram_buffer: u8,

    secondary_oam_idx: usize,
    secondary_oam_data: [u8; SECONDARY_OAM_SIZE],
    sprites: [Sprite; MAX_SPRITES],
    sprite_count: u16,

    // State used during sprite evaluation
    sprite_detected: bool,
    sprite_zero_detected: bool,
    oamaddr_hi: u8,
    oamaddr_lo: u8,
    eval_done: bool,
    overflow_count: u8,

    sprite_zero_visible: bool,

    dma_addr: Option<u16>,

    bg: Tile,
    sprite: Sprite,

    frame: Frame,

    scanline: u16,
    cycle: u16,
    nmi: Cell<bool>,
    prevent_vbl: bool,

    warmup: bool,
    total_cycles: u64,

    bg_addr_buffer: u16,
    spr_addr_buffer: u16,
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
            oam_idx: 0,
            oam_data: [0xff; 256],
            oam_buffer: 0,

            secondary_oam_idx: 0,
            secondary_oam_data: [0xff; 32],
            sprites: [Sprite::default(); MAX_SPRITES],
            sprite_count: 0,

            sprite_detected: false,
            sprite_zero_detected: false,
            oamaddr_hi: 0,
            oamaddr_lo: 0,
            eval_done: false,
            overflow_count: 0,

            sprite_zero_visible: false,

            dma_addr: None,

            bg: Tile::default(),
            sprite: Sprite::default(),

            frame: Frame::new(),

            vram_buffer: 0,

            scanline: 0,
            cycle: 0, // 27 mismatch between cpu and ppu at startup
            nmi: Cell::new(false),
            prevent_vbl: false,

            warmup: true,
            total_cycles: 0,

            bg_addr_buffer: 0,
            spr_addr_buffer: 0,
        }
    }

    pub fn clock(&mut self) {
        self.total_cycles += 1;
        let is_visible_cycle = matches!(self.cycle, VISIBLE_START..=VISIBLE_END);

        match self.scanline {
            VISIBLE_SCANLINE_START..=VISIBLE_SCANLINE_END => {
                self.tick_background();
                self.tick_sprite();

                if self.mask.rendering_enabled() {
                    if is_visible_cycle {
                        self.render_pixel();
                    }
                    self.bg.shift();
                }
            }
            VBLANK_START_LINE if self.cycle == VBLANK_START_CYCLE => self.start_vblank(),
            PRERENDER_LINE => {
                if self.scanline == PRERENDER_LINE {
                    self.end_vblank();
                }
                self.tick_background();
                self.tick_sprite();
            }
            262 => self.scanline = 0,
            _ => {}
        }

        self.cycle += 1;
        if self.cycle > CYCLE_END {
            self.cycle = 0;
            self.scanline += 1;
        }
    }

    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    pub fn load_name_tables(&self, pixels: &mut [u8]) {
        pub const NT_SIZE: u16 = 0x0400;

        for i in 0..4 {
            // x, y normall come from cycle, scanline
            let base_addr = address::NT_START + (i as u16) * NT_SIZE;
            let x_offset = (i % 2) * WIDTH;
            let y_offset = (i / 2) * HEIGHT;

            for addr in base_addr..(base_addr + NT_SIZE - 64) {
                let x_scroll = addr & address::COARSE_X_MASK;
                let y_scroll = (addr & address::COARSE_Y_MASK) >> 5;

                let pattern_table_id = self.bus.peek(addr) as u16;
                let pattern_address = self.control.bg_pattern_addr() | (pattern_table_id << 4);

                let nt_select = addr & (address::NT_X_MASK | address::NT_Y_MASK);
                let attr_base = address::ATTR_START | nt_select; // attribute table start address
                let attr_tile = ((y_scroll & 0xFC) << 1) + (x_scroll >> 2); // hi 3 bits of y, hi 3 bits of x
                let attribute = u16::from(self.bus.peek(attr_base + attr_tile));
                let palette_selector = (x_scroll & 0x02) | ((y_scroll & 0x02) << 1);
                let palette_addr = ((attribute >> palette_selector) & 0x03) << 2;

                let tile_num = x_scroll + (y_scroll << 5);
                let tile_x = (tile_num % 32) << 3;
                let tile_y = (tile_num / 32) << 3;

                for y in 0..8 {
                    let tile_addr = pattern_address + y; // this sets the fine y part of address
                    let tile_lo = self.bus.peek(tile_addr);
                    let tile_hi = self.bus.peek(tile_addr + 8);
                    for x in 0..8 {
                        let tile_palette = ((tile_hi >> x) & 1) << 1 | (tile_lo >> x) & 1;
                        let palette = palette_addr | u16::from(tile_palette);
                        let color = self
                            .bus
                            .peek(PALETTE_START | ((palette & 0x03 > 0) as u16 * palette));
                        let x = u32::from(tile_x + (7 - x));
                        let y = u32::from(tile_y + y);
                        Self::set_pixel(
                            color, // & self.mask.grayscale) | self.mask.emphasis,
                            x + x_offset,
                            y + y_offset,
                            WIDTH,
                            pixels,
                        );
                    }
                }
            }
        }
    }

    pub fn load_pattern_tables(&self, pixels: &mut [u8]) {
        for i in 0..2 {
            let start = (i as u16) * 0x1000;
            let end = start + 0x1000;
            let x_offset = i * WIDTH / 2;

            for tile_addr in (start..end).step_by(16) {
                let tile_x = ((tile_addr % 0x1000) % 256) / 2;
                let tile_y = ((tile_addr % 0x1000) / 256) * 8;
                for y in 0..8 {
                    let tile_lo = self.bus.peek(tile_addr + y) as u16;
                    let tile_hi = self.bus.peek(tile_addr + y + 8) as u16;
                    for x in 0..8 {
                        let palette = (((tile_hi >> x) & 0x01) << 1) | ((tile_lo >> x) & 0x01);
                        let color = self.bus.peek(PALETTE_START | palette);
                        let x = u32::from(tile_x + (7 - x));
                        let y = u32::from(tile_y + y);
                        Self::set_pixel(color, x_offset + x, y, WIDTH, pixels);
                    }
                }
            }
        }
    }

    pub fn load_palette(&self, pixels: &mut [u8]) {
        for addr in PALETTE_START..PALETTE_END {
            let offset = addr - PALETTE_START;
            let x = u32::from(offset % 8) * 32;
            let y = u32::from(offset / 8) * 32;
            let color = self.bus.peek(addr);

            for x in x..x + 32 {
                for y in y..y + 32 {
                    Self::set_pixel(color, x, y, WIDTH, pixels);
                }
            }
        }
    }

    pub fn load_oam(&self, pixels: &mut [u8]) {
        for (i, oamdata) in self.oam_data.chunks(4).enumerate() {
            if let [y, tile_index, attr, x] = oamdata {
                let mut sprite = Sprite {
                    x: *x,
                    y: *y,
                    attribute: *attr,
                    ..Default::default()
                };

                let tile_index = *tile_index as u16;
                let height = self.control.sprite_height();
                let pattern_table = self.control.sprite_pattern_addr();
                let flip_vertical = (attr & 0x80) == 0x80;

                let tile_addr = if height == 8 {
                    pattern_table | (tile_index << 4)
                } else {
                    let pattern_table = (tile_index & 0x1) & 0x1000;
                    pattern_table | ((tile_index & 0xfe) << 4)
                };
                let tile_x = (i % 8) as u32 * 8;
                let tile_y = (i / 8) as u32 * 8;

                for y in 0..8 {
                    let mut y_offset = if flip_vertical { height - 1 - y } else { y };

                    if height == 16 && y_offset >= 8 {
                        y_offset += 8;
                    }

                    sprite.pattern.low = self.bus.peek(tile_addr + y_offset);
                    sprite.pattern.high = self.bus.peek(tile_addr + y_offset + 8);
                    if sprite.attribute & 0x40 == 0x40 {
                        sprite.horizontal_reverse();
                    }

                    let y = u32::from(y);

                    for x in 0..8 {
                        let (palette, color) = sprite.pixel_color();
                        let color = self
                            .bus
                            .peek(PALETTE_START | ((palette << 2) | color) as u16);

                        Self::set_pixel(color, tile_x + x, tile_y + y, 64, pixels);
                        sprite.shift();
                    }
                }
            }
        }
    }

    fn set_pixel(color: u8, x: u32, y: u32, screen_width: u32, pixels: &mut [u8]) {
        let color = 3 * color as usize;
        let idx = 3 * (x + y * screen_width) as usize;

        pixels[idx] = SYSTEM_PALETTE[color];
        pixels[idx + 1] = SYSTEM_PALETTE[color + 1];
        pixels[idx + 2] = SYSTEM_PALETTE[color + 2];
    }

    fn tick_background(&mut self) {
        match self.cycle {
            VISIBLE_START..=VISIBLE_END | BG_PREFETCH_START..=BG_PREFETCH_END => {
                self.fetch_bg();
            }
            COPY_X => {
                if self.mask.rendering_enabled() {
                    self.v_addr.copy_x(self.t_addr);
                }
            }
            COPY_Y_START..=COPY_Y_END
                if self.scanline == PRERENDER_LINE && self.mask.rendering_enabled() =>
            {
                self.v_addr.copy_y(self.t_addr)
            }

            ODD_SKIP_CYCLE => self.cycle = CYCLE_END,
            _ => {}
        }
    }

    fn tick_sprite(&mut self) {
        if self.mask.rendering_enabled() {
            self.shift_sprites();

            match self.cycle {
                // OAM_CLEAR_START..=8 if self.oam_idx >= 0x08 => {
                //     // If OAMADDR is not less than eight when rendering starts, the eight bytes
                //     // starting at OAMADDR & 0xF8 are copied to the first eight bytes of OAM
                // }
                OAM_CLEAR_START..=OAM_CLEAR_END if !self.is_prerender() => {
                    self.clear_secondary_oam();
                }
                SPRITE_EVAL_START..=SPRITE_EVAL_END if !self.is_prerender() => {
                    self.evaluate_sprites()
                }
                SPRITE_FETCH_START..=SPRITE_FETCH_END => self.fetch_sprite(),
                _ => {}
            }
        }
    }

    fn is_prerender(&self) -> bool {
        self.scanline == PRERENDER_LINE
    }

    fn render_pixel(&mut self) {
        let x = self.cycle - 1;
        let y = self.scanline;
        let address = self.v_addr.address();

        // When the PPU isn't rendering, its v register specifies the current VRAM address
        // (and is output on the PPU's address pins). Whenever the low 14 bits of v point into
        // palette RAM ($3F00-$3FFF), the PPU will continuously draw the color at that address
        // instead of the EXT input, overriding the backdrop color.

        // TODO: Does this first check also need to ensure rendering is enabled
        let color = if (address & PALETTE_START) == PALETTE_START {
            self.bus.read(address)
        } else if self.mask.rendering_enabled() {
            let color = self.color_address();
            self.bus.read(color)
        } else {
            // TODO: render_pixel only runs when rendering is enabled, so this will never be hit
            self.bus.read(PALETTE_START)
        };
        self.frame.set_pixel(x, y, color);
    }

    fn color_address(&mut self) -> u16 {
        let show_bg = self.mask.show_background();
        let clip_bg = self.cycle < 8 && !self.mask.show_left_bg();

        let (bg_palette, bg_color) = if show_bg && !clip_bg {
            self.bg.pixel_color(self.fine_x)
        } else {
            (0, 0)
        };

        let show_sprite = self.mask.show_sprite();
        let clip_sprite = self.cycle < 8 && !self.mask.show_left_sprite();

        let sprite = if show_sprite && !clip_sprite {
            if let Some(sprite) = self
                .sprites
                .iter()
                .take(self.sprite_count as usize)
                .find(|sprite| sprite.contains(self.cycle as u8) && sprite.pixel_color().1 != 0)
            {
                sprite
            } else {
                &Sprite::default()
            }
        } else {
            &Sprite::default()
        };
        let (spr_palette, spr_color) = sprite.pixel_color();

        let (palette, color) = match (bg_color, spr_color) {
            (0, 0) => (0, 0),
            (0, _) => (spr_palette, spr_color),
            (_, 0) => (bg_palette, bg_color),
            _ => {
                if sprite.is_sprite_zero && !self.status.sprite_zero_hit() && self.cycle != 255 {
                    self.status.set_sprite_zero_hit();
                }

                if sprite.has_priority() {
                    (spr_palette, spr_color)
                } else {
                    (bg_palette, bg_color)
                }
            }
        };

        PALETTE_START | ((palette << 2) | color) as u16
    }

    /// Fetch 4 tiles and write out shift registers every 8th cycle.
    /// Each tile fetch takes 2 cycles.
    fn fetch_bg(&mut self) {
        match self.cycle & 0x07 {
            1 => {
                self.bg.load_next_tile();
                self.bg_addr_buffer = self.v_addr.nametable_addr();
            }
            2 => {
                self.bg.next_pattern.id = self.bus.read(self.bg_addr_buffer) as u16;
            }
            3 => self.bg_addr_buffer = self.v_addr.attribute_addr(),
            4 => {
                let palette_selector = self.v_addr.attribute_shift(); // determine quadrant in metatile

                self.bg.next_palette = self.bus.read(self.bg_addr_buffer);
                self.bg.next_palette >>= palette_selector;
                self.bg.next_palette &= 0b11;
            }
            5 => {
                self.bg_addr_buffer = self.control.bg_pattern_addr()
                    | (self.bg.next_pattern.id << 4)
                    | self.v_addr.get_fine_y() as u16;
            }
            6 => self.bg.next_pattern.low = self.bus.read(self.bg_addr_buffer) as u16,
            7 => self.bg_addr_buffer += 8,
            _ => {
                self.bg.next_pattern.high = self.bus.read(self.bg_addr_buffer) as u16;

                if self.mask.rendering_enabled() {
                    self.v_addr.increment_x();
                    if self.cycle == INC_Y {
                        self.v_addr.increment_y();
                    }
                }
            }
        }
    }

    fn clear_secondary_oam(&mut self) {
        if self.cycle % 2 == 0 {
            self.secondary_oam_data[self.secondary_oam_idx] = self.oam_buffer;
            self.secondary_oam_idx += 1;
        } else {
            self.oam_buffer = 0xff;
        }
    }

    // https://www.nesdev.org/wiki/PPU_sprite_evaluation#Details
    fn evaluate_sprites(&mut self) {
        if self.cycle == SPRITE_EVAL_START {
            self.sprite_detected = false;
            self.sprite_zero_detected = false; // TODO: does this need locked in at end of eval period??
            self.secondary_oam_idx = 0;
            self.eval_done = false;
            self.oamaddr_hi = 0; // (self.oam_idx >> 2) & 0x3f; // both of these are causing trouble (at max sprites)
            self.oamaddr_lo = 0; // self.oam_idx & 0b11; // but this one is less troublesome
            self.oam_idx = 0; // (self.oamaddr_hi << 2) | self.oamaddr_lo;
        } else if self.cycle == SPRITE_EVAL_END {
            self.sprite_zero_visible = self.sprite_zero_detected;
            self.sprite_count = (self.secondary_oam_idx >> 2) as u16;
        }

        if self.cycle & 0x01 == 0x01 {
            // On odd cycles, data is read from (primary) OAM
            self.oam_buffer = self.oam_data[self.oam_idx as usize];
        } else {
            // On even cycles, data is written to secondary OAM (unless secondary OAM is full, in which case it will read the value in secondary OAM instead)
            let secondary_index = self.secondary_oam_idx & 0x1f;

            if self.eval_done {
                // self.oamaddr_hi = (self.oamaddr_hi + 1) & 0x3f;  // this is causing sprites to miss lines, in conjunction with reset logic above (worse on its own)
                if self.secondary_oam_idx >= SECONDARY_OAM_SIZE {
                    self.oam_buffer = self.secondary_oam_data[secondary_index];
                }
            } else {
                // if not previously detected, interpret this byte as y
                let sprite_y = self.oam_buffer as u16;
                let height = self.control.sprite_height();
                self.sprite_detected |=
                    !self.sprite_detected && (sprite_y..sprite_y + height).contains(&self.scanline);

                if self.secondary_oam_idx < SECONDARY_OAM_SIZE {
                    self.secondary_oam_data[self.secondary_oam_idx] = self.oam_buffer; // this is the even cycle write

                    // If Y-coordinate is in range, copy remaining bytes of sprite data (OAM[n][1] thru OAM[n][3]) into secondary OAM.
                    if self.sprite_detected {
                        self.oamaddr_lo += 1;
                        self.secondary_oam_idx += 1;

                        self.sprite_zero_detected |= self.oamaddr_hi == 0x00;
                        if self.oamaddr_lo == 0x04 {
                            self.sprite_detected = false;
                            self.oamaddr_lo = 0;
                            self.oamaddr_hi = (self.oamaddr_hi + 1) & 0x3f; // max 64 sprites available
                            self.eval_done |= self.oamaddr_hi == 0x00; // cycle around after hitting limit
                        }
                    } else {
                        self.oamaddr_hi = (self.oamaddr_hi + 1) & 0x3f; // this is how we skip 4 bytes if not found
                        self.eval_done |= self.oamaddr_hi == 0x00;
                    }
                } else {
                    // this whole block is doing nothing i think.

                    // secondary oam full, read instead
                    self.oam_buffer = self.secondary_oam_data[secondary_index];

                    if self.sprite_detected {
                        self.status.set_sprite_overflow();

                        self.oamaddr_lo += 1;
                        if self.oamaddr_lo == 0x04 {
                            self.oamaddr_lo = 0;
                            self.oamaddr_hi = (self.oamaddr_hi + 1) & 0x3f;
                            self.eval_done = self.oamaddr_hi == 0x0;

                            // match self.overflow_count.cmp(&0) {
                            //     Ordering::Equal => self.overflow_count = 3,
                            //     Ordering::Greater => {
                            //         self.overflow_count -= 1;
                            //         let no_overflow = self.overflow_count == 0;
                            //         self.eval_done |= no_overflow;
                            //         self.oamaddr_lo *= !no_overflow as u8;
                            //     }
                            //     Ordering::Less => (),
                            // }
                        }
                    } else {
                        // this is sprite overflow bug (broken, causes sprites to disappear)
                        // self.oamaddr_hi = (self.oamaddr_hi + 1) & 0x3f;
                        // self.oamaddr_lo = (self.oamaddr_lo + 1) & 0x03;
                        // self.eval_done |= self.oamaddr_hi == 0x00;
                    }
                }
            }

            self.oam_idx = (self.oamaddr_hi << 2) | (self.oamaddr_lo & 0x03);
        }
    }

    // Load nametables into sprite output unit.  nametable deterined from bytes in secondary oam
    fn fetch_sprite(&mut self) {
        if self.cycle == SPRITE_FETCH_START {
            self.secondary_oam_idx = 0; // TODO: can (should?) this be done at end of eval instead
        }
        let sprite_n = usize::from((self.cycle - SPRITE_FETCH_START) / 8);
        let sprite_index = sprite_n << 2;

        match self.cycle & 0x07 {
            0 => {
                // unused cycle
                self.bg.load_next_tile();
                self.bg_addr_buffer = self.v_addr.nametable_addr();
            }
            1 => {
                // unused cycle
                self.bg.next_pattern.id = self.bus.read(self.bg_addr_buffer) as u16;
            }
            2 => {
                // unused
                self.bg_addr_buffer = self.v_addr.attribute_addr();

                self.sprite.y = self.secondary_oam_data[sprite_index];
                self.sprite.attribute = self.secondary_oam_data[sprite_index + 2];
            }
            3 => {
                // unused
                self.bg.next_palette = self.bus.read(self.bg_addr_buffer);

                self.sprite.x = self.secondary_oam_data[sprite_index + 3];
            }
            4 => self.spr_addr_buffer = self.get_sprite_pattern_address(),
            5 => self.sprite.pattern.low = self.bus.read(self.spr_addr_buffer),
            6 => self.spr_addr_buffer += 8,
            7 => {
                self.sprite.pattern.high = self.bus.read(self.spr_addr_buffer);

                if self.sprite.attribute & 0x40 == 0x40 {
                    self.sprite.horizontal_reverse();
                }
                self.sprite.is_sprite_zero = self.sprite_zero_visible && sprite_n == 0;

                self.sprites[sprite_n] = self.sprite;

                // for spr in self.spr_present.iter_mut().skip(sprite.x as usize).take(8) {
                //     *spr = true;
                // }
            }
            _ => unreachable!(),
        }
    }

    fn shift_sprites(&mut self) {
        if self.cycle > 0 && self.cycle < 256 {
            for i in 0..self.sprite_count as usize {
                if self.sprites[i].contains(self.cycle as u8 - 1) {
                    self.sprites[i].shift();
                }
            }
        }
    }

    // https://www.nesdev.org/wiki/PPU_OAM#Byte_1
    fn get_sprite_pattern_address(&self) -> u16 {
        let sprite_n = (self.cycle - SPRITE_FETCH_START) / 8;
        let oam_idx = (sprite_n as usize) << 2;

        let sprite_y = self.secondary_oam_data[oam_idx] as u16;
        let mut tile_idx = self.secondary_oam_data[oam_idx + 1] as u16;
        let attr = self.secondary_oam_data[oam_idx + 2];
        let height = self.control.sprite_height();
        let pattern_table = self.control.sprite_pattern_addr();
        let flip_vertical = (attr & 0x80) == 0x80;

        let y_offset = if (sprite_y..sprite_y + height).contains(&self.scanline) {
            self.scanline - sprite_y
        } else {
            0
        };
        let mut y_offset = if flip_vertical {
            height - 1 - y_offset
        } else {
            y_offset
        };

        if sprite_n >= self.sprite_count {
            // If there are less than 8 sprites on the next scanline, then dummy fetches to
            // tile $FF occur for the left-over sprites.
            y_offset = 0;
            tile_idx = 0xff
        }

        if height == 8 {
            pattern_table | (tile_idx << 4) | y_offset
        } else {
            let pattern_table = (tile_idx & 0x1) & 0x1000;
            if y_offset >= 8 {
                y_offset += 8; // TODO: this essentially keeps the 4th bit clear.  not sure if this is technically needed.  see tetanes
            }
            pattern_table | ((tile_idx & 0xfe) << 4) | y_offset
        }
    }

    fn start_vblank(&mut self) {
        if !self.prevent_vbl {
            self.status.set_vblank_started();
            if self.control.nmi_enable() {
                self.nmi.set(true);
            }
        }
        self.prevent_vbl = false;
    }

    fn end_vblank(&mut self) {
        self.warmup = false;
        self.status.clear();
        self.nmi.set(false);
    }

    fn vram_increment_addr(&mut self) {
        // TODO: Implement alternate behavior when rendering
        self.v_addr.increment_addr(self.control.vram_increment());
    }

    pub fn clear_dma_addr(&mut self) {
        self.dma_addr = None;
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
        self.oam_idx = value;
    }

    // Register 0x2004 (RW)
    pub fn read_oam_data(&self) -> u8 {
        // TODO: possibly return value from secondary oam during rendering and sprite eval
        self.oam_data[self.oam_idx as usize]
    }

    pub fn write_oam_data(&mut self, value: u8) {
        // TODO: implement glitch on write during render
        // TODO: should only write when not rendering
        self.oam_data[self.oam_idx as usize] = value;
        self.oam_idx = self.oam_idx.wrapping_add(1);
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

    // Register 0x4014 (W)
    pub fn write_dma_addr(&mut self, value: u8) {
        self.dma_addr = Some(value as u16);
    }
}

impl InterruptSignaler for Ppu {
    fn interrupt_status(&self) -> bool {
        let status = self.nmi.get();
        self.nmi.set(false);
        status
    }

    fn dma_status(&self) -> Option<u16> {
        self.dma_addr
    }
}

#[rustfmt::skip]
pub static SYSTEM_PALETTE: &[u8; 0x600] = include_bytes!("Composite_wiki.pal");

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
