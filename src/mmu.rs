use crate::cart::Cartridge;
use crate::rom::ROM;

pub struct MMU {
    rom: ROM,
    cartridge: Box<Cartridge>,
}

impl MMU {
    pub fn new(cartridge: Box<Cartridge>) -> Self {
        Self {
            rom: ROM::boot_rom(),
            cartridge,
        }
    }
}
