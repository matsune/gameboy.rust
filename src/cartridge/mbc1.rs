use super::{ram_size, rom_size, Battery, MBC};
use crate::memory::Memory;

#[derive(Eq, PartialEq)]
enum BankMode {
    Rom,
    Ram,
}

pub struct Mbc1 {
    rom: Vec<u8>,
    rom_bank: u16,
    ram: Vec<u8>,
    ram_bank: u8,
    ram_enabled: bool,
    bank_mode: BankMode,
    battery: Option<Battery>,
}

impl Mbc1 {
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
            bank_mode: BankMode::Rom,
            battery,
        }
    }
}

impl Memory for Mbc1 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3fff => *self.rom.get(usize::from(address)).unwrap_or(&0),
            0x4000..=0x7fff => {
                let a = usize::from(self.rom_bank) * 0x4000 + usize::from(address) - 0x4000;
                *self.rom.get(a).unwrap_or(&0)
            }
            0xa000..=0xbfff => {
                if self.ram_enabled {
                    let bank = match self.bank_mode {
                        BankMode::Rom => 0,
                        BankMode::Ram => u16::from(self.ram_bank),
                    };
                    let a = bank * 0x2000 + address - 0xa000;
                    *self.ram.get(usize::from(a)).unwrap_or(&0)
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
            0x2000..=0x3fff => {
                let n = match value & 0x1f {
                    0 => 1,
                    n => n,
                };
                // TODO: ROM bank wraps around max bank number
                self.rom_bank = (self.rom_bank & 0xe0) | u16::from(n);
            }
            0x4000..=0x5fff => match self.bank_mode {
                BankMode::Rom => self.rom_bank = u16::from(value) & 0x60 | self.rom_bank & 0x1f,
                BankMode::Ram => self.ram_bank = value & 0x03,
            },
            0x6000..=0x7fff => {
                self.bank_mode = if value & 0x01 == 0 {
                    BankMode::Rom
                } else {
                    BankMode::Ram
                };
            }
            0xa000..=0xbfff => {
                if self.ram_enabled {
                    let bank = match self.bank_mode {
                        BankMode::Rom => 0,
                        BankMode::Ram => u16::from(self.ram_bank),
                    };
                    let a = usize::from(bank) * 0x2000 + usize::from(address) - 0xa000;
                    self.ram[a] = value;
                }
            }
            _ => println!("invalid write address {}", address),
        };
    }
}

impl MBC for Mbc1 {}

impl Drop for Mbc1 {
    fn drop(&mut self) {
        if let Some(battery) = &self.battery {
            battery.save_ram(&self.ram);
        }
    }
}
