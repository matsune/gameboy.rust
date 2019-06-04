use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::mmu::MMU;
use std::path::Path;

pub struct Gameboy {
    pub cpu: CPU,
    pub mmu: MMU,
}

impl Gameboy {
    pub fn new<P: AsRef<Path>>(file_path: P, sav_path: Option<P>, skip_boot: bool) -> Self {
        let cartridge = Cartridge::new(file_path, sav_path, skip_boot);
        Self {
            cpu: CPU::new(skip_boot),
            mmu: MMU::new(cartridge, skip_boot),
        }
    }

    pub fn tick(&mut self) {
        let cycles = self.cpu.tick(&mut self.mmu);
        self.mmu.tick(cycles * 4);
    }
}
