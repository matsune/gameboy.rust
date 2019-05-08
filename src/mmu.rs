use crate::cartridge::Cartridge;
use crate::gpu::GPU;
use crate::memory::{Memory, RAM};
use crate::serial::Serial;

pub struct MMU {
    cartridge: Cartridge,
    wram: RAM,
    wram_bank: u8,
    hram: RAM,
    oam: RAM,
    serial: Serial,
    pub gpu: GPU,
    pub interrupt_flag: u8,
    pub interrupt_enable: u8,
}

impl MMU {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,
            wram: RAM::new(0xc000, 0x8000),
            wram_bank: 0,
            hram: RAM::new(0xff80, 0x7f),
            oam: RAM::new(0xfe00, 0xa0),
            serial: Serial::default(),
            gpu: GPU::new(),
            interrupt_flag: 0,
            interrupt_enable: 0,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.gpu.tick(cycles, &mut self.interrupt_flag);
    }
}

impl Memory for MMU {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000...0x7fff => self.cartridge.read(address),
            0x8000...0x9fff => self.gpu.read(address),
            0xa000...0xbfff => self.cartridge.read(address),
            0xc000...0xcfff => self.wram.read(address),
            0xd000...0xdfff => self.wram.read(address + u16::from(self.wram_bank) * 0x1000),
            0xe000...0xfdff => {
                println!("read Reserved Area 0x{:04x}", address);
                0
            }
            0xfe00...0xfe9f => self.oam.read(address),
            0xfea0...0xfeff => {
                println!("read Unused area 0x{:04x}", address);
                0
            }
            0xff00 => unimplemented!("joypad"),
            0xff01...0xff02 => self.serial.read(address),
            0xff04...0xff07 => unimplemented!("timer"),
            0xff0f => self.interrupt_flag,
            0xff40...0xff4f => self.gpu.read(address),
            0xff50 => self.cartridge.read(address),
            0xff51...0xff55 => unimplemented!("hdma"),
            0xff68...0xff6b => self.gpu.read(address),
            0xff70 => self.wram_bank,
            0xff80...0xfffe => self.hram.read(address),
            0xffff => self.interrupt_enable,
            _ => {
                println!("read unknown area 0x{:04x}",address); 
                0
            }
            //_ => unimplemented!("read to address 0x{:04x}", address),
            
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000...0x7fff => self.cartridge.write(address, value),
            0x8000...0x9fff => self.gpu.write(address, value),
            0xa000...0xbfff => self.cartridge.write(address, value),
            0xc000...0xcfff => self.wram.write(address, value),
            0xd000...0xdfff => self
                .wram
                .write(address + u16::from(self.wram_bank) * 0x1000, value),
            0xe000...0xfdff => println!(
                "write to Reserved area 0x{:04x} value 0x{:02x}",
                address, value
            ),
            0xfe00...0xfe9f => self.oam.write(address, value),
            0xfea0...0xfeff => println!(
                "write to Unused area 0x{:04x} value 0x{:02x}",
                address, value
            ),
            0xff00 => unimplemented!("joypad"),
            0xff01...0xff02 => self.serial.write(address, value),
            0xff04...0xff07 => unimplemented!("timer"),
            0xff0f => self.interrupt_flag = value,
            0xff10...0xff3f => {} // sound
            0xff40...0xff4f => self.gpu.write(address, value),
            0xff50 => self.cartridge.write(address, value),
            0xff51...0xff55 => unimplemented!("hdma"),
            0xff68...0xff6b => self.gpu.write(address, value),
            0xff70 => {
                self.wram_bank = match value & 0x07 {
                    0 => 1,
                    n => n,
                };
            }
            0xff80...0xfffe => self.hram.write(address, value),
            0xffff => self.interrupt_enable = value,
            _ => println!(
                "write to unknown area 0x{:04x} value 0x{:02x}",
                address, value
            ),
            // _ => unimplemented!("write to address 0x{:04x} value 0x{:02x}", address, value),
        };
    }
}
