use crate::cartridge::Cartridge;
use crate::gpu::{Hdma, HdmaMode, GPU};
use crate::joypad::{Joypad, JoypadKey};
use crate::serial::Serial;
use crate::sound::{AudioPlayer, Sound};
use crate::timer::Timer;
use crate::util::{get_lsb, get_msb};

pub trait Memory {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);

    fn read_word(&self, address: u16) -> u16 {
        let lsb = self.read(address);
        let msb = self.read(address + 1);
        u16::from(msb) << 8 | u16::from(lsb)
    }

    fn write_word(&mut self, address: u16, value: u16) {
        self.write(address, get_lsb(value));
        self.write(address + 1, get_msb(value));
    }
}

pub struct RAM {
    pub memory: Vec<u8>,
    offset: u16,
}

impl RAM {
    pub fn new(offset: u16, size: u16) -> Self {
        Self {
            memory: vec![0; usize::from(size)],
            offset,
        }
    }
}

impl Memory for RAM {
    fn read(&self, address: u16) -> u8 {
        self.memory[usize::from(address - self.offset)]
    }

    fn write(&mut self, address: u16, value: u8) {
        self.memory[usize::from(address - self.offset)] = value;
    }
}

pub enum InterruptType {
    VBlank = 0,
    LCDC = 1,
    Timer = 2,
    Serial = 3,
    P10P13 = 4,
}

pub struct InterruptFlag {
    inner: u8,
}

impl InterruptFlag {
    pub fn get(&self) -> u8 {
        self.inner
    }

    pub fn interrupt(&mut self, t: InterruptType) {
        self.inner |= 1 << t as u8;
    }
}

impl From<u8> for InterruptFlag {
    fn from(n: u8) -> Self {
        Self { inner: n }
    }
}

pub struct MMU {
    cartridge: Cartridge,
    wram: RAM,
    wram_bank: u8,
    hram: RAM,
    hdma: Hdma,
    serial: Serial,
    timer: Timer,
    joypad: Joypad,
    sound: Option<Sound>,
    pub gpu: GPU,
    pub interrupt_flag: InterruptFlag,
    pub interrupt_enable: u8,
}

impl MMU {
    pub fn new(cartridge: Cartridge, skip_boot: bool) -> Self {
        let is_gbc = cartridge.is_gbc;
        Self {
            cartridge,
            wram: RAM::new(0xc000, 0x8000),
            wram_bank: 0x01,
            hram: RAM::new(0xff80, 0x7f),
            hdma: Hdma::new(),
            serial: Serial::default(),
            timer: Timer::default(),
            joypad: Joypad::default(),
            sound: None,
            gpu: GPU::new(is_gbc, skip_boot),
            interrupt_flag: InterruptFlag::from(0),
            interrupt_enable: 0,
        }
    }

    pub fn enable_sound(&mut self, player: Box<AudioPlayer>) {
        self.sound = Some(Sound::new(player));
    }

    pub fn title(&self) -> &str {
        self.cartridge.title()
    }

    pub fn tick(&mut self, clocks: u32) -> u32 {
        let speed = 1;
        let vram_clocks = self.tick_dma();

        let gpu_clocks = clocks / speed + vram_clocks;
        let cpu_clocks = clocks + vram_clocks * speed;
        self.timer.tick(cpu_clocks, &mut self.interrupt_flag);
        self.gpu.tick(gpu_clocks, &mut self.interrupt_flag);
        if let Some(sound) = &mut self.sound {
            sound.tick(gpu_clocks);
        }

        gpu_clocks
    }

    pub fn keydown(&mut self, key: JoypadKey) {
        self.joypad.keydown(&mut self.interrupt_flag, key);
    }

    pub fn keyup(&mut self, key: JoypadKey) {
        self.joypad.keyup(key);
    }

    fn tick_dma(&mut self) -> u32 {
        if !self.hdma.is_transfer {
            return 0;
        }
        match self.hdma.mode {
            HdmaMode::Gdma => {
                let len = u32::from(self.hdma.len) + 1;
                for _ in 0..len {
                    self.run_dma_hrampart();
                }
                self.hdma.is_transfer = false;
                len * 8
            }
            HdmaMode::Hdma => {
                if !self.gpu.blanked {
                    return 0;
                }
                self.run_dma_hrampart();
                if self.hdma.len == 0x7f {
                    self.hdma.is_transfer = false;
                }
                8
            }
        }
    }

    fn run_dma_hrampart(&mut self) {
        let mmu_src = self.hdma.src;
        for i in 0..0x10 {
            let b = self.read(mmu_src + i);
            self.gpu.write(self.hdma.dst + i, b);
        }
        self.hdma.src += 0x10;
        self.hdma.dst += 0x10;
        if self.hdma.len == 0 {
            self.hdma.len = 0x7f;
        } else {
            self.hdma.len -= 1;
        }
    }
}

impl Memory for MMU {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000...0x7fff => self.cartridge.read(address),
            0x8000...0x9fff => self.gpu.read(address),
            0xa000...0xbfff => self.cartridge.read(address),
            0xc000...0xcfff => self.wram.read(address),
            0xd000...0xdfff => self
                .wram
                .read(address - 0x1000 + u16::from(self.wram_bank) * 0x1000),
            0xe000...0xefff => self.wram.read(address - 0x2000),
            0xf000...0xfdff => self
                .wram
                .read(address - 0x3000 + u16::from(self.wram_bank) * 0x1000),
            0xfe00...0xfe9f => self.gpu.read(address),
            0xff00 => self.joypad.read(address),
            0xff01...0xff02 => self.serial.read(address),
            0xff04...0xff07 => self.timer.read(address),
            0xff0f => self.interrupt_flag.get(),
            0xff10...0xff3f => match &self.sound {
                Some(sound) => sound.read(address),
                None => 0,
            },
            0xff4d => 0, // TODO: speed
            0xff40...0xff45 | 0xff47...0xff4b | 0xff4f => self.gpu.read(address),
            0xff50 => self.cartridge.read(address),
            0xff51...0xff55 => self.hdma.read(address),
            0xff68...0xff6b => self.gpu.read(address),
            0xff70 => self.wram_bank,
            0xff80...0xfffe => self.hram.read(address),
            0xffff => self.interrupt_enable,
            _ => 0,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000...0x7fff => self.cartridge.write(address, value),
            0x8000...0x9fff => self.gpu.write(address, value),
            0xa000...0xbfff => self.cartridge.write(address, value),
            0xc000...0xcfff => self.wram.write(address, value),
            0xd000...0xdfff => self
                .wram
                .write(address - 0x1000 + u16::from(self.wram_bank) * 0x1000, value),
            0xe000...0xefff => self.wram.write(address - 0x2000, value),
            0xf000...0xfdff => self
                .wram
                .write(address - 0x3000 + u16::from(self.wram_bank) * 0x1000, value),
            0xfe00...0xfe9f => self.gpu.write(address, value),
            0xff00 => self.joypad.write(address, value),
            0xff01...0xff02 => self.serial.write(address, value),
            0xff04...0xff07 => self.timer.write(address, value),
            0xff0f => self.interrupt_flag = InterruptFlag::from(value),
            0xff10...0xff3f => match &mut self.sound {
                Some(sound) => sound.write(address, value),
                None => {}
            },
            0xff4d => {} // TODO: shift
            0xff46 => {
                let base = u16::from(value) << 8;
                for i in 0..0xa0 {
                    let b = self.read(base + i);
                    self.write(0xfe00 + i, b);
                }
            }
            0xff40...0xff45 | 0xff47...0xff4b | 0xff4f => self.gpu.write(address, value),
            0xff50 => self.cartridge.write(address, value),
            0xff51...0xff55 => self.hdma.write(address, value),
            0xff68...0xff6b => self.gpu.write(address, value),
            0xff70 => {
                self.wram_bank = match value & 0x07 {
                    0 => 1,
                    n => n,
                };
            }
            0xff80...0xfffe => self.hram.write(address, value),
            0xffff => self.interrupt_enable = value,
            _ => {}
        };
    }
}
