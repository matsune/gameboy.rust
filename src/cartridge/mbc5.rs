use super::{ram_size, rom_size, Battery, MBC};
use crate::memory::Memory;

pub struct Mbc5 {
    rom: Vec<u8>,
    rom_bank: u16,
    ram: Vec<u8>,
    ram_bank: u8,
    ram_enabled: bool,
    battery: Option<Battery>,
}

impl Mbc5 {
    pub fn new(rom: Vec<u8>, battery: Option<Battery>) -> Self {
        let rom_size = rom_size(rom[0x148]);
        let ram_size = ram_size(rom[0x149]);
        assert!(rom_size >= rom.len());
        let ram = match &battery {
            Some(b) => b.load_ram(ram_size),
            None => vec![0u8; ram_size],
        };
        Self {
            rom,
            rom_bank: 1,
            ram,
            ram_bank: 0,
            ram_enabled: false,
            battery,
        }
    }
}

impl Memory for Mbc5 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3fff => *self.rom.get(usize::from(address)).unwrap_or(&0),
            0x4000..=0x7fff => {
                let a = usize::from(self.rom_bank) * 0x4000 + usize::from(address) - 0x4000;
                *self.rom.get(a).unwrap_or(&0)
            }
            0xa000..=0xbfff => {
                if self.ram_enabled {
                    let a = usize::from(self.ram_bank) * 0x2000 + usize::from(address) - 0xa000;
                    *self.ram.get(a).unwrap_or(&0)
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1fff => self.ram_enabled = (value & 0x0f) == 0x0a,
            0x2000..=0x2fff => self.rom_bank = (self.rom_bank & 0xff00) | u16::from(value),
            0x3000..=0x3fff => self.rom_bank = (self.rom_bank & 0xff) | (u16::from(value) << 8),
            0x4000..=0x5fff => self.ram_bank = value & 0x0f,
            0xa000..=0xbfff => {
                if self.ram_enabled {
                    let a = usize::from(self.ram_bank) * 0x2000 + usize::from(address) - 0xa000;
                    self.ram[a] = value;
                }
            }
            _ => println!("invalid write address {}", address),
        };
    }
}

impl MBC for Mbc5 {}

impl Drop for Mbc5 {
    fn drop(&mut self) {
        if let Some(battery) = &self.battery {
            battery.save_ram(&self.ram);
        }
    }
}
