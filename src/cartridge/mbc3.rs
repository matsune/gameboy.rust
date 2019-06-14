use super::{ram_size, rom_size, Battery, MBC};
use crate::memory::Memory;
use std::time::SystemTime;

struct RealTimeClock {
    clock_start: SystemTime,
    latch_start: Option<SystemTime>,
    halt: bool,
    offset_sec: u64,
    halt_seconds: u8,
    halt_minutes: u8,
    halt_hours: u8,
    halt_days: u64,
}

impl Default for RealTimeClock {
    fn default() -> Self {
        Self {
            clock_start: SystemTime::now(),
            latch_start: None,
            halt: false,
            offset_sec: 0,
            halt_seconds: 0,
            halt_minutes: 0,
            halt_hours: 0,
            halt_days: 0,
        }
    }
}

impl RealTimeClock {
    fn latch(&mut self) {
        self.latch_start = Some(SystemTime::now());
    }

    fn unlatch(&mut self) {
        self.latch_start = None;
    }

    fn is_latched(&self) -> bool {
        self.latch_start.is_some()
    }

    fn clock_time_in_secs(&self) -> u64 {
        let now = if let Some(latch) = self.latch_start {
            latch
        } else {
            SystemTime::now()
        };
        let duration = match now.duration_since(self.clock_start) {
            Ok(n) => n.as_secs(),
            Err(_) => 0,
        };
        duration + self.offset_sec
    }

    fn seconds(&self) -> u8 {
        get_seconds(self.clock_time_in_secs())
    }

    fn minutes(&self) -> u8 {
        get_minutes(self.clock_time_in_secs())
    }

    fn hours(&self) -> u8 {
        get_hours(self.clock_time_in_secs())
    }

    fn days(&self) -> u64 {
        get_days(self.clock_time_in_secs())
    }

    fn is_days_overflow(&self) -> bool {
        self.clock_time_in_secs() >= 60 * 60 * 24 * 512
    }

    fn is_halt(&self) -> bool {
        self.halt
    }

    fn set_seconds(&mut self, sec: u8) {
        if !self.halt {
            return;
        }
        self.halt_seconds = sec;
    }

    fn set_minutes(&mut self, min: u8) {
        if !self.halt {
            return;
        }
        self.halt_minutes = min;
    }

    fn set_hours(&mut self, h: u8) {
        if !self.halt {
            return;
        }
        self.halt_hours = h;
    }

    fn set_days(&mut self, days: u64) {
        if !self.halt {
            return;
        }
        self.halt_days = days;
    }

    fn set_halt(&mut self, halt: bool) {
        if halt && !self.halt {
            self.latch();
            self.halt_seconds = self.seconds();
            self.halt_minutes = self.minutes();
            self.halt_hours = self.hours();
            self.halt_days = self.days();
        } else if !halt && self.halt {
            self.offset_sec = u64::from(self.halt_seconds)
                + u64::from(self.halt_minutes) * 60
                + u64::from(self.halt_hours) * 60 * 60
                + self.halt_days * 60 * 60 * 24;
            self.clock_start = SystemTime::now();
        }
        self.halt = halt;
    }

    fn clear_days_overflow(&mut self) {
        while self.is_days_overflow() {
            self.offset_sec -= 60 * 60 * 24 * 512;
        }
    }
}

fn get_seconds(sec: u64) -> u8 {
    (sec % 60) as u8
}
fn get_minutes(sec: u64) -> u8 {
    ((sec % (60 * 60)) / 60) as u8
}
fn get_hours(sec: u64) -> u8 {
    ((sec % (60 * 60 * 24)) / (60 * 60)) as u8
}
fn get_days(sec: u64) -> u64 {
    (sec % (60 * 60 * 24 * 512)) / (60 * 60 * 24)
}

pub struct Mbc3 {
    rom: Vec<u8>,
    rom_bank: u8,
    ram: Vec<u8>,
    ram_bank: u8, // 128 banks
    ram_enabled: bool,
    battery: Option<Battery>,
    rtc: RealTimeClock,
    latch_reg: u8,
}

impl Mbc3 {
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
            rtc: RealTimeClock::default(),
            battery,
            latch_reg: 0xff,
        }
    }

    fn get_timer(&self) -> u8 {
        match self.ram_bank {
            0x08 => self.rtc.seconds(),
            0x09 => self.rtc.minutes(),
            0x0a => self.rtc.hours(),
            0x0b => (self.rtc.days() & 0xff) as u8,
            0x0c => {
                let mut result = (self.rtc.days() & 0x100) >> 8;
                result |= if self.rtc.is_halt() { 1 << 6 } else { 0 };
                result |= if self.rtc.is_days_overflow() {
                    1 << 7
                } else {
                    0
                };
                result as u8
            }
            _ => 0xff,
        }
    }

    fn set_timer(&mut self, value: u8) {
        match self.ram_bank {
            0x08 => self.rtc.set_seconds(value),
            0x09 => self.rtc.set_minutes(value),
            0x0a => self.rtc.set_hours(value),
            0x0b => {
                let day = self.rtc.days();
                self.rtc.set_days((day & 0x100) | u64::from(value));
            }
            0x0c => {
                let day = self.rtc.days();
                self.rtc
                    .set_days(u64::from((day & 0xff) as u16 | u16::from(value & 1) << 8));
                self.rtc.set_halt((value & (1 << 6)) != 0);
                if (value & (1 << 7)) == 0 {
                    self.rtc.clear_days_overflow();
                }
            }
            _ => {}
        }
    }
}

impl Memory for Mbc3 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3fff => *self.rom.get(usize::from(address)).unwrap_or(&0),
            0x4000..=0x7fff => {
                let a = usize::from(self.rom_bank) * 0x4000 + usize::from(address) - 0x4000;
                *self.rom.get(a).unwrap_or(&0)
            }
            0xa000..=0xbfff => {
                if self.ram_bank < 4 {
                    let a = u16::from(self.ram_bank) * 0x2000 + address - 0xa000;
                    *self.ram.get(usize::from(a)).unwrap_or(&0)
                } else {
                    self.get_timer()
                }
            }
            _ => 0,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1fff => self.ram_enabled = (value & 0x0f) == 0x0a,
            0x2000..=0x3fff => {
                self.rom_bank = match value & 0x7f {
                    0 => 1,
                    n => n,
                }
            }
            0x4000..=0x5fff => self.ram_bank = value,
            0x6000..=0x7fff => {
                if value == 0x01 && self.latch_reg == 0 {
                    if self.rtc.is_latched() {
                        self.rtc.unlatch();
                    } else {
                        self.rtc.latch();
                    }
                }
                self.latch_reg = value;
            }
            0xa000..=0xbfff => {
                if self.ram_enabled {
                    if self.ram_bank < 4 {
                        let idx =
                            usize::from(self.ram_bank) * 0x2000 + usize::from(address) - 0xa000;
                        self.ram[idx] = value;
                    } else {
                        self.set_timer(value);
                    }
                }
            }
            _ => println!("invalid write address {}", address),
        };
    }
}
impl MBC for Mbc3 {}

impl Drop for Mbc3 {
    fn drop(&mut self) {
        if let Some(battery) = &self.battery {
            battery.save_ram(&self.ram);
        }
    }
}
