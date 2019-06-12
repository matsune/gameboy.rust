use super::memory::{InterruptFlag, InterruptType, Memory, RAM};
use crate::util::is_bit_on;

pub const SCREEN_W: usize = 160;
pub const SCREEN_H: usize = 144;
const DATA_SIZE: usize = SCREEN_W * SCREEN_H * 3;
type ScreenData = [u8; DATA_SIZE];

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

impl MonoColor {
    // Bit 7-6 - Shade for Color Number 3
    // Bit 5-4 - Shade for Color Number 2
    // Bit 3-2 - Shade for Color Number 1
    // Bit 1-0 - Shade for Color Number 0
    fn new(palette: u8, color_num: u8) -> Self {
        match (palette >> (2 * color_num)) & 0b11 {
            0x00 => MonoColor::White,
            0x01 => MonoColor::Light,
            0x02 => MonoColor::Dark,
            _ => MonoColor::Black,
        }
    }
}

struct Attr {
    below_bg: bool,  // Bit 7
    y_flip: bool,    // Bit 6
    x_flip: bool,    // Bit 5
    is_obp1: bool,   // Bit 4
    bank1: bool,     // Bit 3
    palette_num: u8, // Bit 0-2
}

impl From<u8> for Attr {
    fn from(flags: u8) -> Self {
        Attr {
            below_bg: is_bit_on(flags, 7),
            y_flip: is_bit_on(flags, 6),
            x_flip: is_bit_on(flags, 5),
            is_obp1: is_bit_on(flags, 4),
            bank1: is_bit_on(flags, 3),
            palette_num: flags & 0x07,
        }
    }
}

pub struct GPU {
    is_gbc: bool,
    pub blanked: bool,
    data: ScreenData,
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
            data: [0xff; DATA_SIZE],
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
        self.data.to_vec()
    }

    fn set_color(&mut self, x: usize, a: u8, b: u8, c: u8) {
        let idx = usize::from(self.ly) * SCREEN_W * 3 + x * 3;
        self.data[idx] = a;
        self.data[idx + 1] = b;
        self.data[idx + 2] = c;
    }

    fn set_mono_color(&mut self, x: usize, g: u8) {
        self.set_color(x, g, g, g);
    }

    fn set_rgb_color(&mut self, x: usize, r: u8, g: u8, b: u8) {
        let (r, g, b) = (u32::from(r), u32::from(g), u32::from(b));
        let lr = ((r * 13 + g * 2 + b) >> 1) as u8;
        let lg = ((g * 3 + b) << 1) as u8;
        let lb = ((r * 3 + g * 2 + b * 11) >> 1) as u8;
        self.set_color(x, lr, lg, lb);
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
        if self.lcdc.sprite_enabled() {
            self.draw_sprites();
        }
    }

    fn draw_bg(&mut self) {
        let draw_bg = self.is_gbc || self.lcdc.bg_win_enabled();
        let win_y = if !self.lcdc.window_enabled() || (self.is_gbc && !self.lcdc.bg_win_enabled()) {
            -1
        } else {
            self.ly as i16 - self.wy as i16
        };
        if win_y < 0 && !draw_bg {
            return;
        }
        let tile_base = if self.lcdc.tileset() { 0x8000 } else { 0x8800 };
        let bg_y = self.ly.wrapping_add(self.scy);
        for x in 0..SCREEN_W {
            let win_x = x as i16 - self.wx as i16 + 7;
            let bg_x = x as u16 + u16::from(self.scx);
            let (tilemap_base, tile_y, tile_x, pixel_y, pixel_x) = if win_y >= 0 && win_x >= 0 {
                (
                    if self.lcdc.window_tilemap() {
                        0x9c00
                    } else {
                        0x9800
                    },
                    (win_y as u8 / 8) % 32,
                    (win_x as u8 / 8) % 32,
                    win_y as u8 % 8,
                    win_x as u8 % 8,
                )
            } else {
                (
                    if self.lcdc.bg_tilemap() {
                        0x9c00
                    } else {
                        0x9800
                    },
                    (bg_y as u8 / 8) % 32,
                    (bg_x as u8 / 8) % 32,
                    bg_y as u8 % 8,
                    bg_x as u8 % 8,
                )
            };

            let tile_address = tilemap_base + u16::from(tile_y) * 32 + u16::from(tile_x);
            let tile_num = self.read_ram0(tile_address);
            let tile_location = tile_base
                + (if self.lcdc.tileset() {
                    tile_num as u16
                } else {
                    (tile_num as i8 as i16 + 128) as u16
                }) * 16;

            let attr = Attr::from(self.read_ram1(tile_address));

            let line = u16::from(if self.is_gbc && attr.y_flip {
                (7 - pixel_y) * 2
            } else {
                pixel_y * 2
            });
            let a0 = tile_location + line;
            let (b1, b2) = if self.is_gbc && attr.bank1 {
                (self.read_ram1(a0), self.read_ram1(a0 + 1))
            } else {
                (self.read_ram0(a0), self.read_ram0(a0 + 1))
            };

            let x_bit = if attr.x_flip { pixel_x } else { 7 - pixel_x };
            let c: u8 = if b1 & (1 << x_bit) != 0 { 1 } else { 0 }
                | if b2 & (1 << x_bit) != 0 { 2 } else { 0 };

            self.bg_prio[x] = if c == 0 {
                BgPrio::Color0
            } else if attr.below_bg {
                BgPrio::PrioFlag
            } else {
                BgPrio::Normal
            };

            if self.is_gbc {
                let (r, g, b) = self.bg_palette.get_rgb(attr.palette_num, c);
                self.set_rgb_color(x, r, g, b);
            } else {
                let color = MonoColor::new(self.bgp, c) as u8;
                self.set_mono_color(x, color);
            }
        }
    }

    fn draw_sprites(&mut self) {
        for index in 0..40 {
            let i = 39 - index;
            let address = 0xfe00 + 0x04 * i;
            let pos_y = i32::from(u16::from(self.read(address + 0))) - 16;
            let pos_x = i32::from(u16::from(self.read(address + 1))) - 8;
            let is_8x16 = self.lcdc.sprite_size();
            let pattern_id = self.read(address + 2) & (if is_8x16 { 0xfe } else { 0xff });
            let attr = Attr::from(self.read(address + 3));
            let sprite_height = if is_8x16 { 16 } else { 8 };
            let line = i32::from(self.ly);
            if line < pos_y || line >= pos_y + sprite_height {
                continue;
            }
            if pos_x < -7 || pos_x >= SCREEN_W as i32 + 7 {
                continue;
            }
            let tile_y = if attr.y_flip {
                (sprite_height - 1 - (line - pos_y))
            } else {
                line - pos_y
            } as u16;
            let tile_address = 0x8000 + u16::from(pattern_id) * 0x10 + tile_y * 2;
            let (b1, b2) = if attr.bank1 && self.is_gbc {
                (
                    self.read_ram1(tile_address),
                    self.read_ram1(tile_address + 1),
                )
            } else {
                (
                    self.read_ram0(tile_address),
                    self.read_ram0(tile_address + 1),
                )
            };
            'xloop: for x in 0..8 {
                if pos_x + x < 0 || pos_x + x >= SCREEN_W as i32 {
                    continue;
                }
                let x_bit = 1 << (if attr.x_flip { x } else { 7 - x });
                let c =
                    if (b1 & x_bit) != 0 { 1 } else { 0 } + if (b2 & x_bit) != 0 { 2 } else { 0 };
                if c == 0 {
                    continue;
                }
                if self.is_gbc {
                    if self.lcdc.lcd_enabled()
                        && (self.bg_prio[(pos_x + x) as usize] == BgPrio::PrioFlag
                            || (attr.below_bg
                                && self.bg_prio[(pos_x + x) as usize] != BgPrio::Color0))
                    {
                        continue 'xloop;
                    }
                    let (r, g, b) = self.sprite_palette.get_rgb(attr.palette_num, c);
                    self.set_rgb_color((pos_x + x) as usize, r, g, b);
                } else {
                    if attr.below_bg && self.bg_prio[(pos_x + x) as usize] != BgPrio::Color0 {
                        continue 'xloop;
                    }
                    let color = if attr.is_obp1 {
                        MonoColor::new(self.obp1, c) as u8
                    } else {
                        MonoColor::new(self.obp0, c) as u8
                    };
                    self.set_mono_color((pos_x + x) as usize, color);
                }
            }
        }
    }

    fn read_ram0(&self, address: u16) -> u8 {
        self.ram[0][usize::from(address) - 0x8000]
    }

    fn read_ram1(&self, address: u16) -> u8 {
        self.ram[1][usize::from(address) - 0x8000]
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
                    self.data = [0xff; DATA_SIZE];
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

    fn get_rgb(&self, r: u8, c: u8) -> (u8, u8, u8) {
        (
            self.palettes[usize::from(r)][usize::from(c)][0],
            self.palettes[usize::from(r)][usize::from(c)][1],
            self.palettes[usize::from(r)][usize::from(c)][2],
        )
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
            panic!("Palette unsupported address to read 0x{:04x}", a)
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
            panic!("Palette unsupported address to write 0x{:04x}", a)
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
            _ => panic!("Hdma unsupported address to read 0x{:04x}", a),
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
            _ => panic!("Hdma unsupported address to write 0x{:04x}", a),
        }
    }
}
