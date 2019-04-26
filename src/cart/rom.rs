use crate::cart::Cartridge;

#[derive(Debug)]
pub struct RomOnly {
    memory: Vec<u8>,
}

impl RomOnly {
    pub fn new(memory: Vec<u8>) -> Self {
        Self { memory }
    }
}

impl Cartridge for RomOnly {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000...0x7fff => self.memory[usize::from(address)],
            _ => 0,
        }
    }

    fn write(&mut self, address: u16, value: u8) {}
}
