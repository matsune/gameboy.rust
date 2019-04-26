use crate::cart::lib;
use crate::cart::Cartridge;

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
            self.ram[usize::from(idx)]
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
        let rom_banks = lib::get_rom_banks(memory[0x148]);
        let (ram_banks, bank_size) = lib::get_ram_banks_with_bank_size(memory[0x149]);
        println!(
            "rom_banks {}, ram_banks {}, bank_size {}",
            rom_banks, ram_banks, bank_size
        );
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

impl Cartridge for Mbc1 {
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
                println!("RAM Enable {}", self.ram_enabled);
                if !self.ram_enabled {
                    self.save_ram();
                }
            }
            // Lower ROM Bank Number
            0x2000...0x3fff => {
                let upper = self.selected_rom_bank & 0b01100000;
                let mut lower = u16::from(value) & 0b00011111;
                if lower == 0 {
                    lower = 1;
                }
                self.select_rom_bank(upper | lower);
            }
            // RAM Bank Number or Upper ROM Bank Number
            0x4000...0x5fff => {
                if self.is_rom_mode() {
                    let lower = self.selected_rom_bank & 0b00011111;
                    let upper = u16::from(value) & 0b01100000;
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
