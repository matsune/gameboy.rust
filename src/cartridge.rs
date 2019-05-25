use crate::memory::Memory;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::PathBuf;

const BOOT_ROM: [u8; 0x100] = [
    0x31, 0xFE, 0xFF, 0xAF, 0x21, 0xFF, 0x9F, 0x32, 0xCB, 0x7C, 0x20, 0xFB, 0x21, 0x26, 0xFF, 0x0E,
    0x11, 0x3E, 0x80, 0x32, 0xE2, 0x0C, 0x3E, 0xF3, 0xE2, 0x32, 0x3E, 0x77, 0x77, 0x3E, 0xFC, 0xE0,
    0x47, 0x11, 0x04, 0x01, 0x21, 0x10, 0x80, 0x1A, 0xCD, 0x95, 0x00, 0xCD, 0x96, 0x00, 0x13, 0x7B,
    0xFE, 0x34, 0x20, 0xF3, 0x11, 0xD8, 0x00, 0x06, 0x08, 0x1A, 0x13, 0x22, 0x23, 0x05, 0x20, 0xF9,
    0x3E, 0x19, 0xEA, 0x10, 0x99, 0x21, 0x2F, 0x99, 0x0E, 0x0C, 0x3D, 0x28, 0x08, 0x32, 0x0D, 0x20,
    0xF9, 0x2E, 0x0F, 0x18, 0xF3, 0x67, 0x3E, 0x64, 0x57, 0xE0, 0x42, 0x3E, 0x91, 0xE0, 0x40, 0x04,
    0x1E, 0x02, 0x0E, 0x0C, 0xF0, 0x44, 0xFE, 0x90, 0x20, 0xFA, 0x0D, 0x20, 0xF7, 0x1D, 0x20, 0xF2,
    0x0E, 0x13, 0x24, 0x7C, 0x1E, 0x83, 0xFE, 0x62, 0x28, 0x06, 0x1E, 0xC1, 0xFE, 0x64, 0x20, 0x06,
    0x7B, 0xE2, 0x0C, 0x3E, 0x87, 0xE2, 0xF0, 0x42, 0x90, 0xE0, 0x42, 0x15, 0x20, 0xD2, 0x05, 0x20,
    0x4F, 0x16, 0x20, 0x18, 0xCB, 0x4F, 0x06, 0x04, 0xC5, 0xCB, 0x11, 0x17, 0xC1, 0xCB, 0x11, 0x17,
    0x05, 0x20, 0xF5, 0x22, 0x23, 0x22, 0x23, 0xC9, 0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
    0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
    0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
    0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E, 0x3C, 0x42, 0xB9, 0xA5, 0xB9, 0xA5, 0x42, 0x3C,
    0x21, 0x04, 0x01, 0x11, 0xA8, 0x00, 0x1A, 0x13, 0xBE, 0x00, 0x00, 0x23, 0x7D, 0xFE, 0x34, 0x20,
    0xF5, 0x06, 0x19, 0x78, 0x86, 0x23, 0x05, 0x20, 0xFB, 0x86, 0x00, 0x00, 0x3E, 0x01, 0xE0, 0x50,
];

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

    pub fn has_battery(&self) -> bool {
        match self {
            CartridgeType::Mbc1RamBattery
            | CartridgeType::Mbc2Battery
            | CartridgeType::RomRamBattery
            | CartridgeType::Mmm01SramBattery
            | CartridgeType::Mbc3TimerBattery
            | CartridgeType::Mbc3TimerRamBattery
            | CartridgeType::Mbc3RamBattery
            | CartridgeType::Mbc5RamBattery
            | CartridgeType::Mbc5RumbleSramBattery => true,
            _ => false,
        }
    }
}

trait Battery {
    fn save_path(&self) -> &Option<PathBuf>;
    fn get_ram(&self) -> &Vec<u8>;

    fn save_ram(&self) {
        if let Some(save_path) = self.save_path() {
            match OpenOptions::new()
                .write(true)
                .create(true)
                .open(save_path)
                .and_then(|mut f| f.write_all(self.get_ram()))
            {
                Ok(..) => println!("saved to {:?}", save_path),
                Err(..) => println!("failed to save {:?}", save_path),
            }
        }
    }
}
fn load_ram(save_path: &Option<PathBuf>) -> Option<Vec<u8>> {
    match save_path {
        Some(save_path) => {
            let mut data = vec![];
            match File::open(save_path).and_then(|mut f| f.read_to_end(&mut data)) {
                Ok(..) => Some(data),
                Err(..) => None,
            }
        }
        None => None,
    }
}

pub trait MBC: Memory + Send {}

pub struct Cartridge {
    title: String,
    mbc: Box<MBC>,
    pub skip_boot: bool,
}

impl Cartridge {
    pub fn new(data: Vec<u8>, skip_boot: bool) -> Self {
        let cart_type = CartridgeType::new(data[0x147]);
        println!("CartridgeType: {:?}", cart_type);
        let title = read_title(&data);
        let save_path = if cart_type.has_battery() {
            Option::Some(PathBuf::from(title.clone()).with_extension("sav"))
        } else {
            Option::None
        };
        let mbc: Box<MBC> = if cart_type.is_mbc1() {
            Box::new(Mbc1::new(data, save_path))
        } else if cart_type.is_mbc2() {
            unimplemented!("mbc2")
        } else if cart_type.is_mbc3() {
            unimplemented!("mbc3")
        } else if cart_type.is_mbc5() {
            unimplemented!("mbc5")
        } else {
            Box::new(RomOnly::new(data))
        };
        Cartridge {
            title,
            mbc,
            skip_boot,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}

fn read_title(data: &Vec<u8>) -> String {
    let mut end = 0;
    for i in 0x134..0x142 {
        end = i;
        if data[i] == 0 {
            break;
        }
    }
    String::from_utf8(data[0x134..end].to_vec()).unwrap_or("unknown".to_string())
}

impl Memory for Cartridge {
    fn read(&self, address: u16) -> u8 {
        if self.skip_boot {
            self.mbc.read(address)
        } else {
            match address {
                0x0000...0x00ff => BOOT_ROM[usize::from(address)],
                _ => self.mbc.read(address),
            }
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if !self.skip_boot && address == 0xff50 {
            self.skip_boot = true;
        } else {
            self.mbc.write(address, value)
        }
    }
}

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
            0x0000...0x7fff => self.memory[usize::from(address)],
            _ => 0,
        }
    }

    fn write(&mut self, _address: u16, _value: u8) {}
}

fn rom_size(hex: u8) -> usize {
    0x4000 // 16KB
        * match hex {
            0x00 => 2,
            0x01 => 4,
            0x02 => 8,
            0x03 => 16,
            0x04 => 32,
            0x05 => 64,
            0x06 => 128,
            0x07 => 256,
            0x08 => 512,
            _ => 0,
        }
}

fn ram_size(hex: u8) -> usize {
    match hex {
        0x01 => 0x0800,
        0x02 => 0x2000,
        0x03 => 0x2000 * 4,
        0x04 => 0x2000 * 16,
        0x05 => 0x2000 * 8,
        _ => 0,
    }
}

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
    save_path: Option<PathBuf>,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>, save_path: Option<PathBuf>) -> Self {
        let rom_size = rom_size(rom[0x148]);
        let ram_size = ram_size(rom[0x149]);
        assert!(rom_size >= rom.len());
        let ram = match load_ram(&save_path) {
            Some(data) => data,
            None => vec![0u8; ram_size],
        };
        Self {
            rom,
            rom_bank: 1,
            ram,
            ram_bank: 0,
            ram_enabled: false,
            bank_mode: BankMode::Rom,
            save_path,
        }
    }
}

impl Battery for Mbc1 {
    fn save_path(&self) -> &Option<PathBuf> {
        &self.save_path
    }

    fn get_ram(&self) -> &Vec<u8> {
        &self.ram
    }
}

impl Memory for Mbc1 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000...0x3fff => *self.rom.get(usize::from(address)).unwrap_or(&0),
            0x4000...0x7fff => {
                let a = usize::from(self.rom_bank) * 0x4000 + usize::from(address) - 0x4000;
                *self.rom.get(a).unwrap_or(&0)
            }
            0xa000...0xbfff => {
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
            0x0000...0x1fff => self.ram_enabled = (value & 0x0f) == 0x0a,
            0x2000...0x3fff => {
                let n = match value & 0x1f {
                    0 => 1,
                    n => n,
                };
                // TODO: ROM bank wraps around max bank number
                self.rom_bank = (self.rom_bank & 0xe0) | u16::from(n);
            }
            0x4000...0x5fff => match self.bank_mode {
                BankMode::Rom => self.rom_bank = u16::from(value) & 0x60 | self.rom_bank & 0x1f,
                BankMode::Ram => self.ram_bank = value & 0x03,
            },
            0x6000...0x7fff => {
                self.bank_mode = if value & 0x01 == 0 {
                    BankMode::Rom
                } else {
                    BankMode::Ram
                };
            }
            0xa000...0xbfff => {
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

impl MBC for RomOnly {}
impl MBC for Mbc1 {}

impl Drop for Mbc1 {
    fn drop(&mut self) {
        self.save_ram();
    }
}
