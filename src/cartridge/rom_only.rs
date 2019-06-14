use super::MBC;
use crate::memory::Memory;

#[derive(Debug)]
pub struct RomOnly {
    memory: Vec<u8>,
}

impl RomOnly {
    pub fn new(memory: Vec<u8>) -> Self {
        Self { memory }
    }
}

impl Memory for RomOnly {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7fff => self.memory[usize::from(address)],
            _ => 0,
        }
    }

    fn write(&mut self, _address: u16, _value: u8) {}
}

impl MBC for RomOnly {}
