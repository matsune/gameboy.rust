use crate::cartridge::Cartridge;
use crate::memory::Memory;

pub struct MMU {
    cartridge: Cartridge,
}

impl MMU {
    pub fn new(cartridge: Cartridge) -> Self {
        Self { cartridge }
    }
}

impl Memory for MMU {
    fn read(&self, address: u16) -> u8 {
        if self.cartridge.accepts(address) {
            self.cartridge.read(address)
        } else {
            0
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if self.cartridge.accepts(address) {
            self.cartridge.write(address, value)
        }
    }
}
