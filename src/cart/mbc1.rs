use crate::cart::lib;
use crate::cart::Cartridge;

#[derive(Eq, PartialEq)]
enum Mode {
    Rom,
    Ram,
}

pub struct Mbc1 {
    data: Vec<u8>,
    rom_banks: u16,
    ram_banks: u8,
    selected_rom_bank: u16,
    selected_ram_bank: u8,
    mode: Mode,
    enable_ram_write: bool,
}

impl Mbc1 {
    pub fn new(data: Vec<u8>) -> Self {
        let rom_banks = lib::get_rom_banks(data[0x148]);
        let ram_banks = lib::get_ram_banks(data[0x149]);
        println!("rom_banks {}, ram_banks {}", rom_banks, ram_banks);
        Self {
            data,
            rom_banks,
            ram_banks,
            selected_rom_bank: 1,
            selected_ram_bank: 0,
            mode: Mode::Rom,
            enable_ram_write: true,
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
            u16::from(self.selected_ram_bank % self.ram_banks) * 0x2000 + (address - 0xa000)
        })
    }
}

impl Cartridge for Mbc1 {
    fn read(&self, address: u16) -> u8 {
        // TODO
        0xff
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            // Enable RAM
            0x0000...0x1fff => {
                self.enable_ram_write = (value & 0x0f) == 0x0a;
                println!("RAM Enable {}", self.enable_ram_write);
                if !self.enable_ram_write {
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
            0x6000...0x7fff => {
                self.mode = if value & 0x01 == 0 {
                    Mode::Rom
                } else {
                    Mode::Ram
                };
            }
            0xa000...0xbfff => {
                if self.enable_ram_write {
                    let ram_address = self.get_ram_address(address);
                    if ram_address < self.data.len() {
                        self.data[ram_address] = value;
                    }
                }
            }
            _ => {
                println!("invalid write address {}", address);
            }
        };
    }
}
