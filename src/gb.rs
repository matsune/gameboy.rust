use crate::cart::Cartridge;
use crate::cpu::CPU;

pub struct Gameboy {
    cpu: CPU,
}

impl Gameboy {
    pub fn new(cartridge: Box<Cartridge>) -> Self {
        Self {
            cpu: CPU::new(cartridge),
        }
    }

    pub fn power_on(&mut self) {
        // self.cpu.init();
    }
}
