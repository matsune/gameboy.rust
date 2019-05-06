use std::convert::From;

use crate::memory::{Memory, RAM};
use crate::util::is_bit_on;

pub const PIXELS_W: u8 = 160;
pub const PIXELS_H: u8 = 144;
const T_OAM: usize = 80;
const T_VRAM: usize = 172;
const T_HBLANK: usize = 204;
const T_LINE: usize = T_OAM + T_VRAM + T_HBLANK;
const COLOR_PALETTE: [u8; 4] = [0xff, 0xaa, 0x55, 0x00];

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

#[derive(Debug)]
pub struct GPU {
    vram: RAM,
    mode: Mode,
    cycles: usize,
    lcdc: LCDC,     // 0xff40
    stat: u8,       // 0xff41
    scy: u8,        // 0xff42
    scx: u8,        // 0xff43
    line: u8,       // 0xff44
    lyc: u8,        // 0xff45
    bg_palette: u8, // 0xff47
    pub redraw: bool,
    pub data: Vec<u8>,
}

impl GPU {
    pub fn new() -> Self {
        GPU {
            vram: RAM::new(0x8000, 0x2000),
            lcdc: LCDC::default(),
            mode: Mode::OAM,
            cycles: 0,
            stat: 0,
            scx: 0,
            scy: 0,
            line: 0,
            lyc: 0,
            bg_palette: 0,
            redraw: false,
            data: vec![0u8; usize::from(PIXELS_W) * usize::from(PIXELS_H) * 3],
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.redraw = false;
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
                }
            }
            Mode::HBlank => {
                if self.cycles >= T_HBLANK {
                    self.cycles -= T_HBLANK;
                    self.line += 1;

                    if self.line == PIXELS_H - 1 {
                        self.mode = Mode::VBlank;
                        self.redraw = true;
                    } else {
                        self.mode = Mode::OAM;
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
                    }
                }
            }
        }
    }

    fn get_tile_id(&self, tile_x: u8, tile_y: u8) -> i16 {
        let bg_tilemap_offset: u16 = if self.lcdc.background_tilemap() {
            0x9c00
        } else {
            0x9800
        };
        self.vram
            .read(bg_tilemap_offset + u16::from(tile_x % 32) + u16::from(tile_y % 32) * 32)
            as i16
    }

    fn get_color(&self, tile_id: i16, pixel_x: u8, pixel_y: u8) -> u8 {
        let offset = if self.lcdc.background_tileset() {
            0x8000 + tile_id as u16 * 0x10
        } else {
            if tile_id < 0 {
                0x9000 - (tile_id.abs() as u16) * 0x10
            } else {
                0x9000 + (tile_id.abs() as u16) * 0x10
            }
        };

        let a = self.vram.read(offset + u16::from(pixel_y) * 2);
        let b = self.vram.read(offset + u16::from(pixel_y) * 2 + 1);
        let c = (if is_bit_on(a, 7 - pixel_x) { 1 } else { 0 })
            + (if is_bit_on(b, 7 - pixel_x) { 2 } else { 0 });
        COLOR_PALETTE[usize::from((self.bg_palette >> (c * 2)) & 0x3)]
    }

    fn render_scanline(&mut self) {
        if self.lcdc.background_enabled() {
            let mut tile_x = self.scx / 8;
            let tile_y = self.line.wrapping_add(self.scy) / 8;

            let mut pixel_x = self.scx % 8;
            let pixel_y = self.line.wrapping_add(self.scy) % 8;

            let mut canvas_offset =
                usize::from(PIXELS_W) * 3 * usize::from(self.line) + usize::from(pixel_x) * 3;

            let mut tile_id = self.get_tile_id(tile_x, tile_y);

            for _ in 0..PIXELS_W {
                let color = self.get_color(tile_id, pixel_x, pixel_y);
                self.data[canvas_offset] = color;
                self.data[canvas_offset + 1] = color;
                self.data[canvas_offset + 2] = color;
                canvas_offset += 3;
                pixel_x += 1;

                if pixel_x == 8 {
                    pixel_x = 0;
                    tile_x += 1;
                    tile_id = self.get_tile_id(tile_x, tile_y);
                }
            }
        }
    }
}

impl Memory for GPU {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x8000...0x9fff => self.vram.read(address),
            0xff40 => self.lcdc.get(),
            0xff41 => self.stat,
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.line,
            0xff45 => self.lyc,
            0xff47 => self.bg_palette,
            _ => unimplemented!("GPU read to address 0x{:04x}", address),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x8000...0x9fff => self.vram.write(address, value),
            0xff40 => self.lcdc = LCDC::from(value),
            0xff41 => self.stat = value,
            0xff42 => self.scy = value,
            0xff43 => self.scx = value,
            0xff44 => self.line = value,
            0xff45 => self.lyc = value,
            0xff47 => self.bg_palette = value,
            _ => unimplemented!(
                "GPU write to address 0x{:04x} value 0x{:02x}",
                address,
                value
            ),
        };
    }
}
