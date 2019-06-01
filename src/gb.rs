use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::mmu::MMU;

pub struct Gameboy {
    pub cpu: CPU,
    pub mmu: MMU,
}

impl Gameboy {
    pub fn new(data: Vec<u8>, skip_boot: bool) -> Self {
        let cartridge = Cartridge::new(data, skip_boot, false);
        Self {
            cpu: CPU::new(skip_boot, cartridge.is_gbc),
            mmu: MMU::new(cartridge, skip_boot),
        }
    }

    pub fn tick(&mut self) {
        let is = self.cpu.reg.pc == 0x100;
        let cycles = self.cpu.tick(&mut self.mmu);
        self.mmu.tick(cycles * 4, is);
    }
}
