use crate::cartridge::Cartridge;
use crate::cpu::CPU;

pub struct Gameboy {
    cpu: CPU,
}

impl Gameboy {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cpu: CPU::new(cartridge),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.cpu.run();
        }
    }
}
