use crate::memory::Memory;

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
}

pub trait MBC: Memory + Send {}

pub struct Cartridge {
    mbc: Box<MBC>,
    pub skip_boot: bool,
}

impl Cartridge {
    pub fn new(data: Vec<u8>, skip_boot: bool) -> Self {
        let cart_type = CartridgeType::new(data[0x147]);
        let mbc: Box<MBC> = if cart_type.is_mbc1() {
            Box::new(Mbc1::new(data))
        } else if cart_type.is_mbc2() {
            unimplemented!("mbc2")
        } else if cart_type.is_mbc3() {
            unimplemented!("mbc3")
        } else if cart_type.is_mbc5() {
            unimplemented!("mbc5")
        } else {
            Box::new(RomOnly::new(data))
        };
        Cartridge { mbc, skip_boot }
    }
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

    fn write(&mut self, address: u16, value: u8) {}
}

fn get_rom_banks(hex: u8) -> u16 {
    match hex {
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

fn get_ram_banks_with_bank_size(hex: u8) -> (u8, u16) {
    match hex {
        0x00 => (0, 0),
        0x01 => (1, 2048),
        0x02 => (1, 0x2000),
        0x03 => (4, 0x2000),
        0x04 => (16, 0x2000),
        0x05 => (8, 0x2000),
        _ => (0, 0),
    }
}

#[derive(Debug)]
struct ExternalRam {
    ram: Vec<u8>,
    ram_banks: u8,
    bank_size: u16, // 2KB or 8KB
}

impl ExternalRam {
    fn new(ram_banks: u8, bank_size: u16) -> Self {
        let ram_size = u16::from(ram_banks) * bank_size;
        Self {
            ram: vec![0xff; usize::from(ram_size)],
            ram_banks,
            bank_size,
        }
    }

    fn get_index(&self, offset: u16) -> usize {
        usize::from(u16::from(self.ram_banks) * self.bank_size + offset)
    }

    fn write(&mut self, offset: u16, value: u8) {
        let idx = self.get_index(offset);
        self.ram[idx] = value;
    }

    fn read(&self, offset: u16) -> u8 {
        let idx = self.get_index(offset);
        if idx < self.ram.len() {
            self.ram[idx]
        } else {
            0xff
        }
    }

    fn get_address(&self, bank: u8, offset: u16) -> u16 {
        self.bank_size * u16::from(bank) + offset
    }

    fn save(&self) {
        unimplemented!("save external ram")
    }
}

#[derive(Eq, PartialEq)]
enum Mode {
    Rom,
    Ram,
}

pub struct Mbc1 {
    memory: Vec<u8>,
    rom_banks: u16,
    selected_rom_bank: u16,
    selected_ram_bank: u8,
    mode: Mode,
    ram_enabled: bool,
    ext_ram: ExternalRam,
}

impl Mbc1 {
    pub fn new(memory: Vec<u8>) -> Self {
        let rom_banks = get_rom_banks(memory[0x148]);
        let (ram_banks, bank_size) = get_ram_banks_with_bank_size(memory[0x149]);
        Self {
            memory,
            rom_banks,
            selected_rom_bank: 1,
            selected_ram_bank: 0,
            mode: Mode::Rom,
            ram_enabled: true,
            ext_ram: ExternalRam::new(ram_banks, bank_size),
        }
    }

    fn save_ram(&self) {
        unimplemented!("save ram to battery");
    }

    fn is_rom_mode(&self) -> bool {
        self.mode == Mode::Rom
    }

    fn select_rom_bank(&mut self, bank: u16) {
        self.selected_rom_bank = bank;
    }

    fn select_ram_bank(&mut self, bank: u8) {
        self.selected_ram_bank = bank;
    }

    fn get_ram_address(&self, address: u16) -> usize {
        usize::from(if self.is_rom_mode() {
            address - 0xa000
        } else {
            self.ext_ram
                .get_address(self.selected_ram_bank, address - 0xa000)
        })
    }

    fn get_rom_byte(&self, bank: u16, address: u16) -> u8 {
        let offset = usize::from(bank * 0x4000 + address);
        if offset < self.memory.len() {
            self.memory[offset]
        } else {
            0xff
        }
    }
}

impl Memory for Mbc1 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000...0x3fff => self.get_rom_byte(0, address),
            0x4000...0x7fff => self.get_rom_byte(self.selected_rom_bank, address),
            0xa000...0xbfff => {
                if self.ram_enabled {
                    self.ext_ram.read(address - 0xa000)
                } else {
                    0xff
                }
            }
            _ => {
                println!("invalid write address {}", address);
                0xff
            }
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            // Enable RAM
            0x0000...0x1fff => {
                self.ram_enabled = (value & 0x0f) == 0x0a;
                if !self.ram_enabled {
                    self.save_ram();
                }
            }
            // Lower ROM Bank Number
            0x2000...0x3fff => {
                let upper = self.selected_rom_bank & 0b0110_0000;
                let mut lower = u16::from(value) & 0b0001_1111;
                if lower == 0 {
                    lower = 1;
                }
                self.select_rom_bank(upper | lower);
            }
            // RAM Bank Number or Upper ROM Bank Number
            0x4000...0x5fff => {
                if self.is_rom_mode() {
                    let lower = self.selected_rom_bank & 0b0001_1111;
                    let upper = u16::from(value) & 0b0110_0000;
                    self.select_rom_bank(upper | lower);
                } else {
                    self.select_ram_bank(value & 0b11);
                }
            }
            // Select Banking mode
            0x6000...0x7fff => {
                self.mode = if value & 0x01 == 0 {
                    Mode::Rom
                } else {
                    Mode::Ram
                };
            }
            // Switchable RAM bank
            0xa000...0xbfff => {
                if self.ram_enabled {
                    self.ext_ram.write(address - 0xa000, value);
                }
            }
            _ => println!("invalid write address {}", address),
        };
    }
}

impl MBC for RomOnly {}
impl MBC for Mbc1 {}
