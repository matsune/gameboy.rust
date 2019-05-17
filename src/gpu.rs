use crate::memory::{Memory, RAM};
use crate::mmu::{InterruptFlag, InterruptType};
use crate::util::is_bit_on;
use std::convert::From;

pub const PIXELS_W: u8 = 160;
pub const PIXELS_H: u8 = 144;
pub const DATA_SIZE: usize = (PIXELS_W as usize) * (PIXELS_H as usize) * 3;
const T_OAM: usize = 80;
const T_VRAM: usize = 172;
const T_HBLANK: usize = 204;
const T_LINE: usize = T_OAM + T_VRAM + T_HBLANK;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum BgPrio {
    Color0,
    Normal,
}

enum Color {
    White = 0xff,
    LightGray = 0xc0,
    DarkGray = 0x60,
    Black = 0x00,
}

#[derive(Debug, Default)]
struct LCDC {
    inner: u8,
}

impl LCDC {
    pub fn get(&self) -> u8 {
        self.inner
    }
    pub fn lcd_enabled(&self) -> bool {
        is_bit_on(self.inner, 7)
    }
    pub fn window_tilemap(&self) -> bool {
        is_bit_on(self.inner, 6)
    }
    pub fn window_enabled(&self) -> bool {
        is_bit_on(self.inner, 5)
    }
    pub fn background_tileset(&self) -> bool {
        is_bit_on(self.inner, 4)
    }
    pub fn background_tilemap(&self) -> bool {
        is_bit_on(self.inner, 3)
    }
    pub fn sprite_size(&self) -> bool {
        is_bit_on(self.inner, 2)
    }
    pub fn sprites_enabled(&self) -> bool {
        is_bit_on(self.inner, 1)
    }
    pub fn background_enabled(&self) -> bool {
        is_bit_on(self.inner, 0)
    }
}

impl From<u8> for LCDC {
    fn from(n: u8) -> Self {
        Self { inner: n }
    }
}

#[derive(Debug)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    OAM = 2,
    VRAM = 3,
}

pub struct GPU {
    vram: RAM, // TODO: CGB mode
    mode: Mode,
    cycles: usize,
    oam: RAM,
    lcdc: LCDC, // 0xff40
    stat: u8,   // 0xff41
    scy: u8,    // 0xff42
    scx: u8,    // 0xff43
    line: u8,   // 0xff44
    lyc: u8,    // 0xff45
    // TODO: dma
    dma: u8,        // 0xff46
    bg_palette: u8, // 0xff47
    obp_0: u8,      // 0xff48
    obp_1: u8,      // 0xff49
    wy: u8,         // 0xff4a
    wx: u8,         // 0xff4b
    pub data: [u8; DATA_SIZE],
    bgprio: [BgPrio; PIXELS_W as usize],
}

impl GPU {
    pub fn new() -> Self {
        GPU {
            vram: RAM::new(0x8000, 0x2000),
            lcdc: LCDC::default(),
            mode: Mode::OAM,
            cycles: 0,
            oam: RAM::new(0xfe00, 0xa0),
            stat: 0,
            scx: 0,
            scy: 0,
            line: 0,
            lyc: 0,
            bg_palette: 0,
            dma: 0,
            obp_0: 0x00,
            obp_1: 0x01,
            wy: 0,
            wx: 0,
            data: [0xff; DATA_SIZE],
            bgprio: [BgPrio::Color0; PIXELS_W as usize],
        }
    }

    pub fn tick(&mut self, cycles: usize, interrupt_flag: &mut InterruptFlag) -> bool {
        let mut redraw = false;
        if cycles == 0 {
            return redraw;
        }
        self.cycles += cycles;

        match self.mode {
            Mode::OAM => {
                if self.cycles >= T_OAM {
                    self.mode = Mode::VRAM;
                    self.cycles -= T_OAM;
                }
            }
            Mode::VRAM => {
                if self.cycles >= T_VRAM {
                    self.mode = Mode::HBlank;
                    self.cycles -= T_VRAM;

                    if self.lcdc.lcd_enabled() {
                        self.render_scanline();
                    }
                    if is_bit_on(self.stat, 3) {
                        interrupt_flag.set_flag(InterruptType::LCDC);
                    }
                }
            }
            Mode::HBlank => {
                if self.cycles >= T_HBLANK {
                    self.cycles -= T_HBLANK;
                    self.line += 1;

                    if self.line == PIXELS_H - 1 {
                        self.mode = Mode::VBlank;

                        interrupt_flag.set_flag(InterruptType::VBlank);
                        if is_bit_on(self.stat, 4) {
                            interrupt_flag.set_flag(InterruptType::LCDC);
                        }
                    } else {
                        self.mode = Mode::OAM;
                        if is_bit_on(self.stat, 5) {
                            interrupt_flag.set_flag(InterruptType::LCDC);
                        }
                    }
                }
            }
            Mode::VBlank => {
                if self.cycles >= T_LINE {
                    self.cycles -= T_LINE;
                    self.line += 1;

                    if self.line > PIXELS_H + 10 {
                        self.mode = Mode::OAM;
                        self.line = 0;
                        redraw = true;
                        if is_bit_on(self.stat, 5) {
                            interrupt_flag.set_flag(InterruptType::LCDC);
                        }
                    }
                }
                if is_bit_on(self.stat, 5) && self.line == self.lyc {
                    interrupt_flag.set_flag(InterruptType::LCDC);
                }
            }
        }
        redraw
    }

    fn get_tile_id(&self, tile_x: u8, tile_y: u8) -> i16 {
        let bg_tilemap_offset: u16 = if self.lcdc.background_tilemap() {
            0x9c00
        } else {
            0x9800
        };
        i16::from(
            self.vram
                .read(bg_tilemap_offset + u16::from(tile_x % 32) + u16::from(tile_y % 32) * 32),
        )
    }

    fn get_bg_color(&self, tile_id: i16, pixel_x: u8, pixel_y: u8) -> Color {
        let offset = if self.lcdc.background_tileset() {
            0x8000 + tile_id as u16 * 0x10
        } else if tile_id < 0 {
            0x9000 - (tile_id.abs() as u16) * 0x10
        } else {
            0x9000 + (tile_id.abs() as u16) * 0x10
        };
        let b1 = self.vram.read(offset + u16::from(pixel_y) * 2);
        let b2 = self.vram.read(offset + u16::from(pixel_y) * 2 + 1);
        let c = (if is_bit_on(b1, 7 - pixel_x) { 1 } else { 0 })
            + (if is_bit_on(b2, 7 - pixel_x) { 2 } else { 0 });
        GPU::get_color(self.bg_palette, c)
    }

    fn get_color(palette: u8, c: u8) -> Color {
        let idx = (palette >> (c * 2)) & 0b0000_0011;
        match idx {
            1 => Color::LightGray,
            2 => Color::DarkGray,
            3 => Color::Black,
            _ => Color::White,
        }
    }

    fn reset_window(&mut self) {
        for i in 0..PIXELS_W {
            let offset = usize::from(PIXELS_W) * 3 * usize::from(self.line) + i as usize * 3;
            self.data[offset + 0] = 0xff;
            self.data[offset + 1] = 0xff;
            self.data[offset + 2] = 0xff;
        }
    }

    fn draw_bg(&mut self) {
        let mut tile_x = self.scx / 8;
        let tile_y = self.line.wrapping_add(self.scy) / 8;

        let mut pixel_x = self.scx % 8;
        let pixel_y = self.line.wrapping_add(self.scy) % 8;

        let mut canvas_offset =
            usize::from(PIXELS_W) * 3 * usize::from(self.line) + usize::from(pixel_x) * 3;

        let mut tile_id = self.get_tile_id(tile_x, tile_y);

        for _ in 0..PIXELS_W {
            let color = self.get_bg_color(tile_id, pixel_x, pixel_y) as u8;
            self.data[canvas_offset] = color;
            self.data[canvas_offset + 1] = color;
            self.data[canvas_offset + 2] = color;
            canvas_offset += 3;
            pixel_x += 1;
            self.bgprio[pixel_x as usize] = if color == 0 {
                BgPrio::Color0
            } else {
                BgPrio::Normal
            };
            if pixel_x == 8 {
                pixel_x = 0;
                tile_x += 1;
                tile_id = self.get_tile_id(tile_x, tile_y);
            }
        }
    }

    fn draw_sprites(&mut self) {
        for index in 0..40 {
            let i = 39 - index;
            let address = 0xfe00 + 0x04 * i;
            let pos_y = self.read(address + 0) as u16 as i32 - 16;
            let pos_x = self.read(address + 1) as u16 as i32 - 8;
            let is_8x16_size = self.lcdc.sprite_size();
            let pattern_id = self.read(address + 2) & (if is_8x16_size { 0xfe } else { 0xff });
            let flags = self.read(address + 3);
            let below_bg = is_bit_on(flags, 7);
            let is_flip_y = is_bit_on(flags, 6);
            let is_flip_x = is_bit_on(flags, 5);
            let is_obp1 = is_bit_on(flags, 4);
            let sprite_height = if is_8x16_size { 16 } else { 8 };
            let line = self.line as i32;
            if line < pos_y || line >= pos_y + sprite_height {
                continue;
            }
            if pos_x < -7 || pos_x >= PIXELS_W as i32 + 7 {
                continue;
            }
            let tile_y = if is_flip_y {
                (sprite_height - 1 - (line - pos_y))
            } else {
                line - pos_y
            } as u16;
            let tile_address = 0x8000 + pattern_id as u16 * 0x10 + tile_y * 2;
            let (b1, b2) = (
                self.vram.read(tile_address),
                self.vram.read(tile_address + 1),
            );
            for x in 0..8 {
                if pos_x + x < 0 || pos_x + x >= PIXELS_W as i32 {
                    continue;
                }
                let x_bit = 1 << (if is_flip_x { x } else { 7 - x });
                let c =
                    if (b1 & x_bit) != 0 { 1 } else { 0 } + if (b2 & x_bit) != 0 { 2 } else { 0 };
                if c == 0 {
                    continue;
                }
                if below_bg && self.bgprio[(pos_x + x) as usize] != BgPrio::Color0 {
                    continue;
                }
                let color = if is_obp1 {
                    GPU::get_color(self.obp_1, c) as u8
                } else {
                    GPU::get_color(self.obp_0, c) as u8
                };
                let canvas_offset =
                    usize::from(PIXELS_W) * 3 * usize::from(self.line) + ((pos_x + x) * 3) as usize;
                self.data[canvas_offset + 0] = color;
                self.data[canvas_offset + 1] = color;
                self.data[canvas_offset + 2] = color;
            }
        }
    }

    fn render_scanline(&mut self) {
        self.reset_window();
        if self.lcdc.background_enabled() {
            self.draw_bg();
        }
        if self.lcdc.sprites_enabled() {
            self.draw_sprites();
        }
    }
}

impl Memory for GPU {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x8000...0x9fff => self.vram.read(address),
            0xfe00...0xfe9f => self.oam.read(address),
            0xff40 => self.lcdc.get(),
            0xff41 => self.stat,
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.line,
            0xff45 => self.lyc,
            0xff46 => 0,
            0xff47 => self.bg_palette,
            0xff48 => self.obp_0,
            0xff49 => self.obp_1,
            0xff4a => self.wy,
            0xff4b => self.wx,
            _ => unimplemented!("GPU read to address 0x{:04x}", address),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x8000...0x9fff => {
                self.vram.write(address, value);
            }
            0xfe00...0xfe9f => self.oam.write(address, value),
            0xff40 => self.lcdc = LCDC::from(value),
            0xff41 => self.stat = value,
            0xff42 => self.scy = value,
            0xff43 => self.scx = value,
            0xff44 => {}
            0xff45 => self.lyc = value,
            0xff46 => {}
            0xff47 => self.bg_palette = value,
            0xff48 => self.obp_0 = value,
            0xff49 => self.obp_1 = value,
            0xff4a => self.wy = value,
            0xff4b => self.wx = value,
            _ => unimplemented!(
                "GPU write to address 0x{:04x} value 0x{:02x}",
                address,
                value
            ),
        };
    }
}
