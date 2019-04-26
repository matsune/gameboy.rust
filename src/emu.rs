use crate::cart::Cartridge;
use crate::gb::Gameboy;

pub struct Emulator {
    cartridge: Box<Cartridge>,
    gameboy: Gameboy,
}

impl Emulator {
    pub fn new(memory: Vec<u8>) -> Self {
        Self {
            cartridge: crate::cart::load_cartridge(memory),
            gameboy: Gameboy::new(),
        }
    }

    pub fn run(&self) {}
}
