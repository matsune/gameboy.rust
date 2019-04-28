use crate::cart::Cartridge;
use crate::mmu::MMU;
use crate::reg::Registers;

pub struct CPU {
    registers: Registers,
    mmu: MMU,
}

impl CPU {
    pub fn new(cartridge: Box<Cartridge>) -> Self {
        Self {
            registers: Registers::default(),
            mmu: MMU::new(cartridge),
        }
    }
}
