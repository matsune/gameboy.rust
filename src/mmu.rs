use crate::cartridge::Cartridge;
use crate::gpu::GPU;
use crate::joypad::{Joypad, JoypadKey};
use crate::memory::{Memory, RAM};
use crate::serial::Serial;
use crate::timer::Timer;

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
    serial: Serial,
    timer: Timer,
    joypad: Joypad,
    pub gpu: GPU,
    pub interrupt_flag: InterruptFlag,
    pub interrupt_enable: u8,
}

impl MMU {
    pub fn new(cartridge: Cartridge, skip_boot: bool) -> Self {
        Self {
            cartridge,
            wram: RAM::new(0xc000, 0x8000),
            wram_bank: 0x01,
            hram: RAM::new(0xff80, 0x7f),
            serial: Serial::default(),
            timer: Timer::default(),
            joypad: Joypad::default(),
            gpu: GPU::new(skip_boot),
            interrupt_flag: InterruptFlag::from(0),
            interrupt_enable: 0,
        }
    }

    pub fn title(&self) -> &str {
        self.cartridge.title()
    }

    pub fn tick(&mut self, clocks: usize) {
        self.timer.tick(clocks, &mut self.interrupt_flag);
        self.gpu.tick(clocks, &mut self.interrupt_flag);
    }

    pub fn keydown(&mut self, key: JoypadKey) {
        self.joypad.keydown(&mut self.interrupt_flag, key);
    }

    pub fn keyup(&mut self, key: JoypadKey) {
        self.joypad.keyup(key);
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
            0xff10...0xff3f => 0, // sound
            0xff4d => 0,          // TODO: speed
            0xff40...0xff4f => self.gpu.read(address),
            0xff50 => self.cartridge.read(address),
            0xff51...0xff55 => unimplemented!("hdma"),
            0xff68...0xff6b => self.gpu.read(address),
            0xff70 => self.wram_bank,
            0xff80...0xfffe => self.hram.read(address),
            0xffff => self.interrupt_enable,
            _ => {
                println!("read unknown area 0x{:04x}", address);
                0
            }
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
            0xff10...0xff3f => {} // TODO: sound
            0xff4d => {}          // TODO: shift
            0xff46 => {
                let base = u16::from(value) << 8;
                for i in 0..0xa0 {
                    let b = self.read(base + i);
                    self.write(0xfe00 + i, b);
                }
            }
            0xff40...0xff4f => self.gpu.write(address, value),
            0xff50 => self.cartridge.write(address, value),
            0xff51...0xff55 => unimplemented!("hdma"),
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
