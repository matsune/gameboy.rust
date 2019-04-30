use crate::cartridge::Cartridge;
use crate::gb::Gameboy;

pub struct Emulator {
    gameboy: Gameboy,
}

impl Emulator {
    pub fn new(memory: Vec<u8>) -> Self {
        Self {
            gameboy: Gameboy::new(Cartridge::new(memory)),
        }
    }

    pub fn run(&mut self) {
        self.gameboy.run();
    }
}
