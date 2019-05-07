use crate::cartridge::Cartridge;
use crate::gpu::GPU;
use crate::memory::{Memory, RAM};

pub struct MMU {
    cartridge: Cartridge,
    hram: RAM,
    pub gpu: GPU,
    pub interrupt_flag: u8,
    pub interrupt_enable: u8,
}

impl MMU {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,
            hram: RAM::new(0xff80, 0x7f),
            gpu: GPU::new(),
            interrupt_flag: 0,
            interrupt_enable: 0,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.gpu.tick(cycles, &mut self.interrupt_flag);
    }
}

impl Memory for MMU {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000...0x7fff => self.cartridge.read(address),
            0x8000...0x9fff => self.gpu.read(address),
            0xa000...0xbfff => self.cartridge.read(address),
            0xff0f => self.interrupt_flag,
            0xff40...0xff4f => self.gpu.read(address),
            0xff50 => self.cartridge.read(address),
            0xff80...0xfffe => self.hram.read(address),
            0xffff => self.interrupt_enable,
            _ => unimplemented!("read to address 0x{:04x}", address),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000...0x7fff => self.cartridge.write(address, value),
            0x8000...0x9fff => self.gpu.write(address, value),
            0xa000...0xbfff => self.cartridge.write(address, value),
            0xff0f => self.interrupt_flag = value,
            0xff10...0xff26 => {} // sound
            0xff40...0xff4f => self.gpu.write(address, value),
            0xff50 => self.cartridge.write(address, value),
            0xff80...0xfffe => self.hram.write(address, value),
            0xffff => self.interrupt_enable = value,
            _ => unimplemented!("write to address 0x{:04x} value 0x{:02x}", address, value),
        };
    }
}
