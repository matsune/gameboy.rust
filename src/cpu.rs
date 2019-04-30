use crate::cartridge::Cartridge;
use crate::memory::Memory;
use crate::mmu::MMU;
use crate::reg::Registers;

pub struct CPU {
    registers: Registers,
    mmu: MMU,
}

impl CPU {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            registers: Registers::default(),
            mmu: MMU::new(cartridge),
        }
    }

    pub fn run(&mut self) -> u8 {
        self.handle_interrupt();
        let opcode = self.read_byte();
        let cycles = self.operate(opcode);
        println!("{:?}", self.registers);
        cycles
    }

    fn read_byte(&mut self) -> u8 {
        let mut pc = self.registers.get_pc();
        let opcode = self.mmu.read(pc);
        self.registers.set_pc(pc + 1);
        opcode
    }

    fn read_word(&mut self) -> u16 {
        let mut pc = self.registers.get_pc();
        let opcode = u16::from(self.mmu.read(pc)) | u16::from(self.mmu.read(pc + 1)) << 8;
        self.registers.set_pc(pc + 2);
        opcode
    }

    fn handle_interrupt(&self) {
        // TODO:
    }

    fn operate(&mut self, opcode: u8) -> u8 {
        match opcode {
            0x06 => {
                let n = self.read_byte();
                self.registers.set_b(n);
                println!("0x{:x} LD B, 0x{:x}", opcode, n);
                8
            }
            0x31 => {
                let n = self.read_word();
                self.registers.set_sp(n);
                println!("0x{:x} LD SP, 0x{:x}", opcode, n);
                12
            }
            _ => panic!("unknown instruction {}", opcode),
        }
    }
}
