mod lib;
mod mbc1;
mod rom;

use mbc1::Mbc1;
use rom::RomOnly;

#[derive(Debug)]
pub enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc1Ram,
    Mbc1RamBattery,
    Mbc2,
    Mbc2Battery,
    RomRam,
    RomRamBattery,
    Mmm01,
    Mmm01Sram,
    Mmm01SramBattery,
    Mbc3TimerBattery,
    Mbc3TimerRamBattery,
    Mbc3,
    Mbc3Ram,
    Mbc3RamBattery,
    Mbc5,
    Mbc5Ram,
    Mbc5RamBattery,
    Mbc5Rumble,
    Mbc5RumbleSram,
    Mbc5RumbleSramBattery,
}

impl CartridgeType {
    pub fn new(n: u8) -> CartridgeType {
        match n {
            0x00 => CartridgeType::RomOnly,
            0x01 => CartridgeType::Mbc1,
            0x02 => CartridgeType::Mbc1Ram,
            0x03 => CartridgeType::Mbc1RamBattery,
            0x05 => CartridgeType::Mbc2,
            0x06 => CartridgeType::Mbc2Battery,
            0x08 => CartridgeType::RomRam,
            0x09 => CartridgeType::RomRamBattery,
            0x0b => CartridgeType::Mmm01,
            0x0c => CartridgeType::Mmm01Sram,
            0x0d => CartridgeType::Mmm01SramBattery,
            0x0f => CartridgeType::Mbc3TimerBattery,
            0x10 => CartridgeType::Mbc3TimerRamBattery,
            0x11 => CartridgeType::Mbc3,
            0x12 => CartridgeType::Mbc3Ram,
            0x13 => CartridgeType::Mbc3RamBattery,
            0x19 => CartridgeType::Mbc5,
            0x1a => CartridgeType::Mbc5Ram,
            0x1b => CartridgeType::Mbc5RamBattery,
            0x1c => CartridgeType::Mbc5Rumble,
            0x1d => CartridgeType::Mbc5RumbleSram,
            0x1e => CartridgeType::Mbc5RumbleSramBattery,
            _ => panic!("invalid CartridgeType hex {}", n),
        }
    }

    pub fn is_mbc1(&self) -> bool {
        match self {
            CartridgeType::Mbc1 | CartridgeType::Mbc1Ram | CartridgeType::Mbc1RamBattery => true,
            _ => false,
        }
    }

    pub fn is_mbc2(&self) -> bool {
        match self {
            CartridgeType::Mbc2 | CartridgeType::Mbc2Battery => true,
            _ => false,
        }
    }

    pub fn is_mbc3(&self) -> bool {
        match self {
            CartridgeType::Mbc3TimerBattery
            | CartridgeType::Mbc3TimerRamBattery
            | CartridgeType::Mbc3
            | CartridgeType::Mbc3Ram
            | CartridgeType::Mbc3RamBattery => true,
            _ => false,
        }
    }

    pub fn is_mbc5(&self) -> bool {
        match self {
            CartridgeType::Mbc5
            | CartridgeType::Mbc5Ram
            | CartridgeType::Mbc5RamBattery
            | CartridgeType::Mbc5Rumble
            | CartridgeType::Mbc5RumbleSram
            | CartridgeType::Mbc5RumbleSramBattery => true,
            _ => false,
        }
    }
}

pub trait Cartridge {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

pub fn load_cartridge(memory: Vec<u8>) -> Box<Cartridge> {
    let cart_type = CartridgeType::new(memory[0x147]);
    if cart_type.is_mbc1() {
        Box::new(Mbc1::new(memory))
    } else if cart_type.is_mbc2() {
        unimplemented!("mbc2");
    } else if cart_type.is_mbc3() {
        unimplemented!("mbc3");
    } else if cart_type.is_mbc5() {
        unimplemented!("mbc5");
    } else {
        Box::new(RomOnly::new(memory))
    }
}
