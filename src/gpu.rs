use super::memory::{Memory, RAM};
use crate::mmu::{InterruptFlag, InterruptType};
use crate::util::is_bit_on;

pub const SCREEN_W: usize = 160;
pub const SCREEN_H: usize = 144;

#[derive(PartialEq, Copy, Clone)]
enum BgPrio {
    Color0,
    PrioFlag,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum StatMode {
    HBlank = 0,
    VBlank = 1,
    OAM = 2,
    VRAM = 3,
}

pub enum MonoColor {
    White = 0xff,
    Light = 0xc0,
    Dark = 0x60,
    Black = 0x00,
}

pub struct GPU {
    is_gbc: bool,
    pub blanked: bool,
    data: [[[u8; 3]; SCREEN_W]; SCREEN_H],
    pub redraw: bool,
    bgp: u8,
    clocks: usize,
    lcdc: Lcdc,
    ly: u8,
    ly_compare: u8,
    oam: RAM,
    obp0: u8,
    obp1: u8,
    ram: [[u8; 0x2000]; 0x02],
    ram_bank: usize,
    scx: u8,
    scy: u8,
    stat: Stat,
    wx: u8,
    wy: u8,
    bg_prio: [BgPrio; SCREEN_W],
    bg_palette: Palette,
    sprite_palette: Palette,
}

impl GPU {
    pub fn new(is_gbc: bool, skip_boot: bool) -> Self {
        let (lcdc, bgp) = if skip_boot {
            (Lcdc::from(0x91), 0xfc)
        } else {
            (Lcdc::from(0x48), 0x00)
        };
        Self {
            is_gbc,
            blanked: false,
            data: [[[0xffu8; 3]; SCREEN_W]; SCREEN_H],
            redraw: false,
            bgp,
            clocks: 0,
            lcdc,
            ly: 0x00,
            ly_compare: 0x00,
            oam: RAM::new(0xfe00, 0xa0),
            obp0: 0xff,
            obp1: 0xff,
            ram: [[0u8; 0x2000]; 0x02],
            ram_bank: 0,
            scx: 0x00,
            scy: 0x00,
            stat: Stat::default(),
            wx: 0x00,
            wy: 0x00,
            bg_prio: [BgPrio::Normal; SCREEN_W],
            bg_palette: Palette::new(0xff68),
            sprite_palette: Palette::new(0xff6a),
        }
    }

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

    fn set_color(&mut self, x: usize, r: u8, g: u8, b: u8) {
        let (r, g, b) = (u32::from(r), u32::from(g), u32::from(b));
        let lr = ((r * 13 + g * 2 + b) >> 1) as u8;
        let lg = ((g * 3 + b) << 1) as u8;
        let lb = ((r * 3 + g * 2 + b * 11) >> 1) as u8;
        self.data[usize::from(self.ly)][x] = [lr, lg, lb];
    }

    pub fn tick(&mut self, clocks: usize, int_flag: &mut InterruptFlag) {
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

            let tile_address = usize::from(tilemapbase + tiley * 32 + tilex);
            let tile_num = self.ram[0][tile_address - 0x8000];
            let tilebase = if self.lcdc.tileset() { 0x8000 } else { 0x8800 };
            let tile_location = tilebase
                + (if self.lcdc.tileset() {
                    tile_num as u16
                } else {
                    (tile_num as i8 as i16 + 128) as u16
                }) * 16;
            let flags = self.ram[1][tile_address - 0x8000];
            let below_bg = is_bit_on(flags, 7);
            let is_flip_y = is_bit_on(flags, 6);
            let is_flip_x = is_bit_on(flags, 5);
            let rambank_1 = is_bit_on(flags, 3);
            let palette_num_1 = flags & 0x07;

            let line: u16 = if self.is_gbc && is_flip_y {
                (7 - pixely) * 2
            } else {
                pixely * 2
            };
            let a0 = usize::from(tile_location + line);
            let (b1, b2) = if self.is_gbc && rambank_1 {
                (self.ram[1][a0 - 0x8000], self.ram[1][a0 + 1 - 0x8000])
            } else {
                (self.ram[0][a0 - 0x8000], self.ram[0][a0 + 1 - 0x8000])
            };

            let x_bit = if is_flip_x { pixelx } else { 7 - pixelx };
            let c = if b1 & (1 << x_bit) != 0 { 1 } else { 0 }
                | if b2 & (1 << x_bit) != 0 { 2 } else { 0 };

            self.bg_prio[x] = if c == 0 {
                BgPrio::Color0
            } else if below_bg {
                BgPrio::PrioFlag
            } else {
                BgPrio::Normal
            };

            if self.is_gbc {
                let r = self.bg_palette.get(usize::from(palette_num_1), c, 0);
                let g = self.bg_palette.get(usize::from(palette_num_1), c, 1);
                let b = self.bg_palette.get(usize::from(palette_num_1), c, 2);
                self.set_color(x, r, g, b);
            } else {
                let color = Self::get_mono_color(self.bgp, c) as u8;
                self.set_mono_color(x, color);
            }
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
            let rambank_1 = is_bit_on(flags, 3);
            let palette_num_1 = flags & 0x07;
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
            let (b1, b2) = if rambank_1 && self.is_gbc {
                (
                    self.ram[1][usize::from(tile_address) - 0x8000],
                    self.ram[1][usize::from(tile_address) + 1 - 0x8000],
                )
            } else {
                (
                    self.ram[0][usize::from(tile_address) - 0x8000],
                    self.ram[0][usize::from(tile_address) + 1 - 0x8000],
                )
            };
            'xloop: for x in 0..8 {
                if pos_x + x < 0 || pos_x + x >= SCREEN_W as i32 {
                    continue;
                }
                let x_bit = 1 << (if is_flip_x { x } else { 7 - x });
                let c =
                    if (b1 & x_bit) != 0 { 1 } else { 0 } + if (b2 & x_bit) != 0 { 2 } else { 0 };
                if c == 0 {
                    continue;
                }
                if self.is_gbc {
                    if self.lcdc.lcd_enabled()
                        && (self.bg_prio[(pos_x + x) as usize] == BgPrio::PrioFlag
                            || (below_bg && self.bg_prio[(pos_x + x) as usize] != BgPrio::Color0))
                    {
                        continue 'xloop;
                    }
                    let r = self.sprite_palette.get(usize::from(palette_num_1), c, 0);
                    let g = self.sprite_palette.get(usize::from(palette_num_1), c, 1);
                    let b = self.sprite_palette.get(usize::from(palette_num_1), c, 2);
                    self.set_color((pos_x + x) as usize, r, g, b);
                } else {
                    if below_bg && self.bg_prio[(pos_x + x) as usize] != BgPrio::Color0 {
                        continue 'xloop;
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
    }
}

impl Memory for GPU {
    fn read(&self, a: u16) -> u8 {
        match a {
            0x8000...0x9fff => self.ram[self.ram_bank][usize::from(a - 0x8000)],
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
            0xff4f => self.ram_bank as u8,
            0xff68 | 0xff69 => self.bg_palette.read(a),
            _ => panic!("Unsupported address to read 0x{:04x}", a),
        }
    }

    fn write(&mut self, a: u16, v: u8) {
        match a {
            0x8000...0x9fff => self.ram[self.ram_bank][usize::from(a - 0x8000)] = v,
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
            0xff47 => self.bgp = v,
            0xff48 => self.obp0 = v,
            0xff49 => self.obp1 = v,
            0xff4a => self.wy = v,
            0xff4b => self.wx = v,
            0xff4f => self.ram_bank = usize::from(v & 0x01),
            0xff68 | 0xff69 => self.bg_palette.write(a, v),
            0xff6a | 0xff6b => self.sprite_palette.write(a, v),
            _ => panic!("Unsupported address to write 0x{:04x}", a),
        }
    }
}

struct Palette {
    offset: u16,
    index: u8,
    increment: bool,
    palettes: [[[u8; 3]; 4]; 8],
}

impl Palette {
    fn new(offset: u16) -> Self {
        Palette {
            offset,
            index: 0,
            increment: false,
            palettes: [[[0; 3]; 4]; 8],
        }
    }

    fn get(&self, r: usize, c: usize, idx: usize) -> u8 {
        self.palettes[r][c][idx]
    }
}

impl Memory for Palette {
    fn read(&self, a: u16) -> u8 {
        if a == self.offset {
            self.index | (if self.increment { 0x80 } else { 0x00 })
        } else if a == (self.offset + 1) {
            let r = usize::from(self.index) / 8;
            let c = usize::from(self.index / 2) % 4;
            if self.index & 0x01 == 0 {
                let a = self.palettes[r][c][0];
                let b = self.palettes[r][c][1] << 5;
                a | b
            } else {
                let a = self.palettes[r][c][1] >> 3;
                let b = self.palettes[r][c][2] << 2;
                a | b
            }
        } else {
            panic!()
        }
    }

    fn write(&mut self, a: u16, v: u8) {
        if a == self.offset {
            self.index = v & 0x3f;
            self.increment = is_bit_on(v, 7);
        } else if a == (self.offset + 1) {
            let r = usize::from(self.index) / 8;
            let c = usize::from(self.index / 2) % 4;
            if self.index & 0x01 == 0 {
                self.palettes[r][c][0] = v & 0x1f;
                self.palettes[r][c][1] = (self.palettes[r][c][1] & 0x18) | (v >> 5);
            } else {
                self.palettes[r][c][1] = (self.palettes[r][c][1] & 0x07) | ((v & 0x03) << 3);
                self.palettes[r][c][2] = (v >> 2) & 0x1f;
            }
            if self.increment {
                self.index = (self.index + 1) & 0x3f;
            }
        } else {
            panic!()
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum HdmaMode {
    Gdma,
    Hdma,
}

pub struct Hdma {
    pub data: [u8; 4],
    pub mode: HdmaMode,
    pub is_transfer: bool,
    pub len: u8,
    pub src: u16,
    pub dst: u16,
}

impl Hdma {
    pub fn new() -> Self {
        Hdma {
            data: [0; 4],
            mode: HdmaMode::Gdma,
            is_transfer: false,
            len: 0x00,
            src: 0x00,
            dst: 0x00,
        }
    }
}

impl Memory for Hdma {
    fn read(&self, a: u16) -> u8 {
        match a {
            0xff51...0xff54 => self.data[usize::from(a) - 0xff51],
            0xff55 => self.len | (if self.is_transfer { 0 } else { 1 << 7 }),
            _ => panic!("Hdma read"),
        }
    }

    fn write(&mut self, a: u16, v: u8) {
        match a {
            0xff51...0xff54 => self.data[usize::from(a) - 0xff51] = v,
            0xff55 => {
                if self.is_transfer && self.mode == HdmaMode::Hdma {
                    if !is_bit_on(v, 7) {
                        self.is_transfer = false;
                    }
                    return;
                }
                self.is_transfer = true;
                self.mode = if is_bit_on(v, 7) {
                    HdmaMode::Hdma
                } else {
                    HdmaMode::Gdma
                };
                self.len = v & 0x7f;
                self.src = (u16::from(self.data[0]) << 8) | u16::from(self.data[1]);
                self.dst = (u16::from(self.data[2]) << 8) | u16::from(self.data[3]) | 0x8000;
            }
            _ => panic!("Hdma write"),
        }
    }
}
