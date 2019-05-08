use crate::cartridge::Cartridge;
use crate::cpu::CPU;

use crate::mmu::MMU;

pub struct Gameboy {
    pub cpu: CPU,
    pub mmu: MMU,
}

impl Gameboy {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cpu: CPU::new(),
            mmu: MMU::new(cartridge),
        }
    }

    pub fn tick(&mut self) -> bool {
        let cycles = self.cpu.tick(&mut self.mmu);
        self.mmu.tick(cycles)
    }
}
