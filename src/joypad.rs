use crate::memory::Memory;
use crate::util::is_bit_on;

pub enum JoypadKey {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Start,
    Select,
}

pub struct Joypad {
    p1: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad { p1: 0 }
    }
}

impl Memory for Joypad {
    fn read(&self, address: u16) -> u8 {
        self.p1
    }

    fn write(&mut self, address: u16, value: u8) {
        if address == 0xff00 {
            self.p1 = value & 0b0011_0000;
            if is_bit_on(self.p1, 4) {
                self.p1 |= 0x0f;
            }
            if is_bit_on(self.p1, 5) {
                self.p1 |= 0x0f;
            }
        }
    }
}
