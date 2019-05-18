use super::memory::{Memory, RAM};
use crate::mmu::{InterruptFlag, InterruptType};
use crate::util::is_bit_on;

pub const SCREEN_W: usize = 160;
pub const SCREEN_H: usize = 144;

#[derive(PartialEq, Copy, Clone)]
enum BgPrio {
    Color0,
    Normal,
}

struct Lcdc {
    inner: u8,
}

impl std::convert::From<u8> for Lcdc {
    fn from(n: u8) -> Self {
        Self { inner: n }
    }
}

impl Default for Lcdc {
    fn default() -> Self {
        Self { inner: 0b0100_1000 }
    }
}

impl Lcdc {
    fn get(&self) -> u8 {
        self.inner
    }

    fn lcd_enabled(&self) -> bool {
        is_bit_on(self.inner, 7)
    }

    fn window_tilemap(&self) -> bool {
        is_bit_on(self.inner, 6)
    }

    fn window_enabled(&self) -> bool {
        is_bit_on(self.inner, 5)
    }

    fn tileset(&self) -> bool {
        is_bit_on(self.inner, 4)
    }

    fn bg_tilemap(&self) -> bool {
        is_bit_on(self.inner, 3)
    }

    fn sprite_size(&self) -> bool {
        is_bit_on(self.inner, 2)
    }

    fn sprite_enabled(&self) -> bool {
        is_bit_on(self.inner, 1)
    }

    fn bg_win_enabled(&self) -> bool {
        is_bit_on(self.inner, 0)
    }
}

struct Stat {
    ly_interrupt_enabled: bool,
    oam_interrupt_enabled: bool,
    vblank_interrupt_enabled: bool,
    hblank_interrupt_enabled: bool,
    mode: StatMode,
}

impl Default for Stat {
    fn default() -> Self {
        Self {
            ly_interrupt_enabled: false,
            oam_interrupt_enabled: false,
            vblank_interrupt_enabled: false,
            hblank_interrupt_enabled: false,
            mode: StatMode::HBlank,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum StatMode {
    HBlank = 0,
    VBlank = 1,
    OAM = 2,
    VRAM = 3,
}

impl StatMode {
    fn clocks(&self) -> usize {
        match self {
            StatMode::HBlank => 204,
            StatMode::VBlank => 456,
            StatMode::OAM => 80,
            StatMode::VRAM => 172,
        }
    }
}

pub enum MonoColor {
    White = 0xff,
    Light = 0xc0,
    Dark = 0x60,
    Black = 0x00,
}

pub struct GPU {
    pub blanked: bool,
    data: [[[u8; 3]; SCREEN_W]; SCREEN_H],
    pub redraw: bool,
    bgp: u8,
    clocks: u32,
    lcdc: Lcdc,
    ly: u8,
    ly_compare: u8,
    oam: RAM,
    obp0: u8,
    obp1: u8,
    vram: RAM,
    scx: u8,
    scy: u8,
    stat: Stat,
    wx: u8,
    wy: u8,
    bg_prio: [BgPrio; SCREEN_W],
}

impl Default for GPU {
    fn default() -> Self {
        Self {
            blanked: false,
            data: [[[0xffu8; 3]; SCREEN_W]; SCREEN_H],
            redraw: false,
            bgp: 0x00,
            clocks: 0,
            lcdc: Lcdc::default(),
            ly: 0x00,
            ly_compare: 0x00,
            oam: RAM::new(0xfe00, 0xa0),
            obp0: 0x00,
            obp1: 0x01,
            vram: RAM::new(0x8000, 0x2000),
            scx: 0x00,
            scy: 0x00,
            stat: Stat::default(),
            wx: 0x00,
            wy: 0x00,
            bg_prio: [BgPrio::Normal; SCREEN_W],
        }
    }
}

impl GPU {
    pub fn get_rgb_data(&self) -> Vec<u8> {
        let mut res = Vec::new();
        for line in self.data.iter() {
            for rgb in line.iter() {
                res.extend(rgb);
            }
        }
        res
    }

    fn get_mono_color(v: u8, i: usize) -> MonoColor {
        match (v >> 2 * i) & 0x03 {
            0x00 => MonoColor::White,
            0x01 => MonoColor::Light,
            0x02 => MonoColor::Dark,
            _ => MonoColor::Black,
        }
    }

    fn set_mono_color(&mut self, x: usize, g: u8) {
        self.data[usize::from(self.ly)][x] = [g, g, g];
    }

    pub fn tick(&mut self, clocks: u32, int_flag: &mut InterruptFlag) {
        if !self.lcdc.lcd_enabled() {
            return;
        }
        self.blanked = false;

        if clocks == 0 {
            return;
        }
        let c = (clocks - 1) / 80 + 1;
        for i in 0..c {
            self.clocks += if i == c - 1 { clocks % 80 } else { 80 };
            if self.clocks >= 456 {
                self.clocks -= 456;
                self.ly = (self.ly + 1) % 154;
                if self.stat.ly_interrupt_enabled && self.ly == self.ly_compare {
                    int_flag.interrupt(InterruptType::LCDC);
                }

                if self.ly >= 144 && self.stat.mode != StatMode::VBlank {
                    self.set_mode(StatMode::VBlank, int_flag);
                }
            }

            if self.ly < 144 {
                if self.clocks <= 80 {
                    if self.stat.mode != StatMode::OAM {
                        self.set_mode(StatMode::OAM, int_flag);
                    }
                } else if self.clocks <= 252 {
                    if self.stat.mode != StatMode::VRAM {
                        self.set_mode(StatMode::VRAM, int_flag);
                    }
                } else {
                    if self.stat.mode != StatMode::HBlank {
                        self.set_mode(StatMode::HBlank, int_flag);
                    }
                }
            }
        }
    }

    fn set_mode(&mut self, mode: StatMode, int_flag: &mut InterruptFlag) {
        self.stat.mode = mode;
        let interrupts = match mode {
            StatMode::HBlank => {
                self.render_scan();
                self.blanked = true;
                self.stat.hblank_interrupt_enabled
            }
            StatMode::VBlank => {
                self.redraw = true;
                int_flag.interrupt(InterruptType::VBlank);
                self.stat.vblank_interrupt_enabled
            }
            StatMode::OAM => self.stat.oam_interrupt_enabled,
            _ => false,
        };
        if interrupts {
            int_flag.interrupt(InterruptType::LCDC);
        }
    }

    fn render_scan(&mut self) {
        for x in 0..SCREEN_W {
            self.set_mono_color(x, 0xff);
            self.bg_prio[x] = BgPrio::Normal;
        }
        self.draw_bg();
        self.draw_sprites();
    }

    fn draw_bg(&mut self) {
        let drawbg = false || self.lcdc.bg_win_enabled();

        let winy = if !self.lcdc.window_enabled() || (false && !self.lcdc.bg_win_enabled()) {
            -1
        } else {
            self.ly as i32 - self.wy as i32
        };

        if winy < 0 && drawbg == false {
            return;
        }

        let wintiley = (winy as u16 >> 3) & 31;

        let bgy = self.scy.wrapping_add(self.ly);
        let bgtiley = (bgy as u16 >> 3) & 31;

        for x in 0..SCREEN_W {
            let winx = -((self.wx as i32) - 7) + (x as i32);
            let bgx = self.scx as u32 + x as u32;

            let (tilemapbase, tiley, tilex, pixely, pixelx) = if winy >= 0 && winx >= 0 {
                (
                    if self.lcdc.window_tilemap() {
                        0x9c00
                    } else {
                        0x9800
                    },
                    wintiley,
                    (winx as u16 >> 3),
                    winy as u16 & 0x07,
                    winx as u8 & 0x07,
                )
            } else if drawbg {
                (
                    if self.lcdc.bg_tilemap() {
                        0x9C00
                    } else {
                        0x9800
                    },
                    bgtiley,
                    (bgx as u16 >> 3) & 31,
                    bgy as u16 & 0x07,
                    bgx as u8 & 0x07,
                )
            } else {
                continue;
            };

            let tilenr: u8 = self.vram.read(tilemapbase + tiley * 32 + tilex);

            let tilebase = if self.lcdc.tileset() { 0x8000 } else { 0x8800 };
            let tileaddress = tilebase
                + (if tilebase == 0x8000 {
                    tilenr as u16
                } else {
                    (tilenr as i8 as i16 + 128) as u16
                }) * 16;

            let a0 = tileaddress + (pixely * 2);

            let (b1, b2) = (self.vram.read(a0), self.vram.read(a0 + 1));

            let xbit = 7 - pixelx as u32;
            let colnr = if b1 & (1 << xbit) != 0 { 1 } else { 0 }
                | if b2 & (1 << xbit) != 0 { 2 } else { 0 };

            self.bg_prio[x] = if colnr == 0 {
                BgPrio::Color0
            } else {
                BgPrio::Normal
            };

            let color = Self::get_mono_color(self.bgp, colnr) as u8;
            self.set_mono_color(x, color);
        }
    }

    fn draw_sprites(&mut self) {
        if !self.lcdc.sprite_enabled() {
            return;
        }

        for index in 0..40 {
            let i = 39 - index;
            let address = 0xfe00 + 0x04 * i;
            let pos_y = i32::from(u16::from(self.read(address + 0))) - 16;
            let pos_x = i32::from(u16::from(self.read(address + 1))) - 8;
            let is_8x16 = self.lcdc.sprite_size();
            let pattern_id = self.read(address + 2) & (if is_8x16 { 0xfe } else { 0xff });
            let flags = self.read(address + 3);
            let below_bg = is_bit_on(flags, 7);
            let is_flip_y = is_bit_on(flags, 6);
            let is_flip_x = is_bit_on(flags, 5);
            let is_obp1 = is_bit_on(flags, 4);
            let sprite_height = if is_8x16 { 16 } else { 8 };
            let line = i32::from(self.ly);
            if line < pos_y || line >= pos_y + sprite_height {
                continue;
            }
            if pos_x < -7 || pos_x >= SCREEN_W as i32 + 7 {
                continue;
            }
            let tile_y = if is_flip_y {
                (sprite_height - 1 - (line - pos_y))
            } else {
                line - pos_y
            } as u16;
            let tile_address = 0x8000 + u16::from(pattern_id) * 0x10 + tile_y * 2;
            let (b1, b2) = (
                self.vram.read(tile_address),
                self.vram.read(tile_address + 1),
            );
            for x in 0..8 {
                if pos_x + x < 0 || pos_x + x >= SCREEN_W as i32 {
                    continue;
                }
                let x_bit = 1 << (if is_flip_x { x } else { 7 - x });
                let c =
                    if (b1 & x_bit) != 0 { 1 } else { 0 } + if (b2 & x_bit) != 0 { 2 } else { 0 };
                if c == 0 {
                    continue;
                }
                if below_bg && self.bg_prio[(pos_x + x) as usize] != BgPrio::Color0 {
                    continue;
                }
                let color = if is_obp1 {
                    Self::get_mono_color(self.obp1, c) as u8
                } else {
                    Self::get_mono_color(self.obp0, c) as u8
                };
                self.set_mono_color((pos_x + x) as usize, color);
            }
        }
    }

    //  fn draw_sprites(&mut self) {
    //      if !self.lcdc.sprite_enabled() {
    //          return;
    //      }

    //      for index in 0..40 {
    //          let i = 39 - index;
    //          let spriteaddr = 0xFE00 + (i as u16) * 4;
    //          let spritey = self.read(spriteaddr + 0) as u16 as i32 - 16;
    //          let spritex = self.read(spriteaddr + 1) as u16 as i32 - 8;
    //          let tilenum = (self.read(spriteaddr + 2)
    //              & (if self.lcdc.sprite_size() { 0xFE } else { 0xFF }))
    //              as u16;
    //          let flags = self.read(spriteaddr + 3) as usize;
    //          let usepal1: bool = flags & (1 << 4) != 0;
    //          let xflip: bool = flags & (1 << 5) != 0;
    //          let yflip: bool = flags & (1 << 6) != 0;
    //          let belowbg: bool = flags & (1 << 7) != 0;
    //          let c_palnr = flags & 0x07;
    //          let c_vram1: bool = flags & (1 << 3) != 0;

    //          let line = self.ly as i32;
    //          let sprite_size = if self.lcdc.sprite_size() { 16 } else { 8 };

    //          if line < spritey || line >= spritey + sprite_size {
    //              continue;
    //          }
    //          if spritex < -7 || spritex >= (SCREEN_W as i32) {
    //              continue;
    //          }

    //          let tiley: u16 = if yflip {
    //              (sprite_size - 1 - (line - spritey)) as u16
    //          } else {
    //              (line - spritey) as u16
    //          };

    //          let tileaddress = 0x8000u16 + tilenum * 16 + tiley * 2;
    //          let (b1, b2) = (self.vram.read(tileaddress), self.vram.read(tileaddress + 1));

    //          'xloop: for x in 0..8 {
    //              if spritex + x < 0 || spritex + x >= (SCREEN_W as i32) {
    //                  continue;
    //              }

    //              let xbit = 1 << (if xflip { x } else { 7 - x } as u32);
    //              let colnr =
    //                  (if b1 & xbit != 0 { 1 } else { 0 }) | (if b2 & xbit != 0 { 2 } else { 0 });
    //              if colnr == 0 {
    //                  continue;
    //              }

    //              if belowbg && self.bg_prio[(spritex + x) as usize] != BgPrio::Color0 {
    //                  continue 'xloop;
    //              }
    //              let color = if usepal1 {
    //                  Self::get_mono_color(self.obp1, colnr) as u8
    //              } else {
    //                  Self::get_mono_color(self.obp0, colnr) as u8
    //              };
    //              self.set_mono_color((spritex + x) as usize, color);
    //          }
    //      }
    //  }
}

impl Memory for GPU {
    fn read(&self, a: u16) -> u8 {
        match a {
            0x8000...0x9fff => self.vram.read(a),
            0xfe00...0xfe9f => self.oam.read(a),
            0xff40 => self.lcdc.get(),
            0xff41 => {
                let bit6 = if self.stat.ly_interrupt_enabled {
                    0x40
                } else {
                    0x00
                };
                let bit5 = if self.stat.oam_interrupt_enabled {
                    0x20
                } else {
                    0x00
                };
                let bit4 = if self.stat.vblank_interrupt_enabled {
                    0x10
                } else {
                    0x00
                };
                let bit3 = if self.stat.hblank_interrupt_enabled {
                    0x08
                } else {
                    0x00
                };
                let bit2 = if self.ly == self.ly_compare {
                    0x04
                } else {
                    0x00
                };
                bit6 | bit5 | bit4 | bit3 | bit2 | self.stat.mode as u8
            }
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.ly,
            0xff45 => self.ly_compare,
            0xff46 => 0,
            0xff47 => self.bgp,
            0xff48 => self.obp0,
            0xff49 => self.obp1,
            0xff4a => self.wy,
            0xff4b => self.wx,
            _ => panic!("Unsupported address"),
        }
    }

    fn write(&mut self, a: u16, v: u8) {
        match a {
            0x8000...0x9fff => self.vram.write(a, v),
            0xfe00...0xfe9f => self.oam.write(a, v),
            0xff40 => {
                self.lcdc = v.into();
                if !self.lcdc.lcd_enabled() {
                    self.clocks = 0;
                    self.ly = 0;
                    self.stat.mode = StatMode::HBlank;
                    self.data = [[[0xffu8; 3]; SCREEN_W]; SCREEN_H];
                    self.redraw = true;
                }
            }
            0xff41 => {
                self.stat.ly_interrupt_enabled = is_bit_on(v, 6);
                self.stat.oam_interrupt_enabled = is_bit_on(v, 5);
                self.stat.vblank_interrupt_enabled = is_bit_on(v, 4);
                self.stat.hblank_interrupt_enabled = is_bit_on(v, 3);
            }
            0xff42 => self.scy = v,
            0xff43 => self.scx = v,
            0xff44 => {}
            0xff45 => self.ly_compare = v,
            0xff46 => {}
            0xff47 => self.bgp = v,
            0xff48 => self.obp0 = v,
            0xff49 => self.obp1 = v,
            0xff4a => self.wy = v,
            0xff4b => self.wx = v,
            _ => panic!("Unsupported address"),
        }
    }
}
