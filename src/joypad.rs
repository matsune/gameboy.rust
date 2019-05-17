use crate::memory::Memory;
use crate::mmu::{InterruptFlag, InterruptType};
use crate::util::is_bit_on;

pub enum JoypadKey {
    Right = 1,
    Left = 1 << 1,
    Up = 1 << 2,
    Down = 1 << 3,
    A = 1 << 4,
    B = 1 << 5,
    Select = 1 << 6,
    Start = 1 << 7,
}

pub struct Joypad {
    p1: u8,
    matrix: u8,
}

impl Default for Joypad {
    fn default() -> Self {
        Joypad {
            p1: 0,
            matrix: 0xff,
        }
    }
}

impl Joypad {
    pub fn keydown(&mut self, interrupt_flag: &mut InterruptFlag, key: JoypadKey) {
        self.matrix &= !(key as u8);
        interrupt_flag.set_flag(InterruptType::P10P13);
    }

    pub fn keyup(&mut self, key: JoypadKey) {
        self.matrix |= key as u8;
    }
}

impl Memory for Joypad {
    fn read(&self, address: u16) -> u8 {
        assert_eq!(address, 0xff00);
        if !is_bit_on(self.p1, 4) {
            self.p1 | (self.matrix & 0x0f)
        } else if !is_bit_on(self.p1, 5) {
            self.p1 | (self.matrix >> 4)
        } else {
            self.p1
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        assert_eq!(address, 0xff00);
        self.p1 = value;
    }
}
