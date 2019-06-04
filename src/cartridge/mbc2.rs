use super::{load_ram, ram_size, rom_size, Battery, MBC};
use crate::memory::Memory;
use std::path::PathBuf;

pub struct Mbc2 {
    rom: Vec<u8>,
    rom_bank: u8,
    ram: Vec<u8>,
    ram_enabled: bool,
    save_path: Option<PathBuf>,
}

impl Mbc2 {
    pub fn new(rom: Vec<u8>, save_path: Option<PathBuf>) -> Self {
        let rom_size = rom_size(rom[0x148]);
        let ram_size = 512;
        assert!(rom_size >= rom.len());
        let ram = match load_ram(&save_path) {
            Some(data) => data,
            None => vec![0u8; ram_size],
        };
        Self {
            rom,
            rom_bank: 1,
            ram,
            ram_enabled: false,
            save_path,
        }
    }
}

impl Battery for Mbc2 {
    fn save_path(&self) -> &Option<PathBuf> {
        &self.save_path
    }

    fn get_ram(&self) -> &Vec<u8> {
        &self.ram
    }
}

impl Memory for Mbc2 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000...0x3fff => *self.rom.get(usize::from(address)).unwrap_or(&0),
            0x4000...0x7fff => {
                let a = usize::from(self.rom_bank) * 0x4000 + usize::from(address) - 0x4000;
                *self.rom.get(a).unwrap_or(&0)
            }
            0xa000...0xbfff => {
                if self.ram_enabled {
                    *self.ram.get(usize::from(address) - 0xa000).unwrap_or(&0)
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000...0x1fff => {
                if (address >> 8) & 0x01 == 0 {
                    self.ram_enabled = !self.ram_enabled;
                }
            }
            0x2000...0x3fff => {
                if (address >> 8) & 0x01 == 1 {
                    self.rom_bank = value & 0x0f;
                }
            }
            0xa000...0xbfff => {
                if self.ram_enabled {
                    self.ram[usize::from(address) - 0xa000] = value;
                }
            }
            _ => println!("invalid write address {}", address),
        };
    }
}

impl MBC for Mbc2 {}

impl Drop for Mbc2 {
    fn drop(&mut self) {
        self.save_ram();
    }
}
