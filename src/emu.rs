use crate::cart;
use crate::cart::Cartridge;
use crate::gb::Gameboy;

pub struct Emulator {
    gameboy: Gameboy,
}

impl Emulator {
    pub fn new(memory: Vec<u8>) -> Self {
        Self {
            gameboy: Gameboy::new(cart::load_cartridge(memory)),
        }
    }

    pub fn run(&self) {}
}
