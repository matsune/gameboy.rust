use crate::memory::Memory;
use crate::mmu::{InterruptFlag, InterruptType};
use crate::util::is_bit_on;

pub struct Timer {
    div: Counter,
    tima: Counter,
    tma: u8,
    tac: u8,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            div: Counter::new(16384),
            tima: Counter::new(4096),
            tma: 0,
            tac: 0,
        }
    }
}

impl Timer {
    fn set_tima(&mut self) {
        match self.tma & 0b11 {
            0b00 => self.tima = Counter::new(4096),
            0b01 => self.tima = Counter::new(262_144),
            0b10 => self.tima = Counter::new(65536),
            0b11 => self.tima = Counter::new(16384),
            _ => panic!(),
        }
    }

    pub fn tick(&mut self, interrupt_flag: &mut InterruptFlag) {
        self.div.tick();
        if is_bit_on(self.tac, 2) {
            let updated = self.tima.tick();
            if updated && self.tima.counter() == 0 {
                self.tima.set_counter(self.tma);
                interrupt_flag.interrupt(InterruptType::Timer);
            }
        }
    }
}

impl Memory for Timer {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xff04 => self.div.counter(),
            0xff05 => self.tima.counter(),
            0xff06 => self.tma,
            0xff07 => self.tac,
            _ => panic!("invalid address {:04x} Timer", address),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0xff04 => self.div.reset(),
            0xff05 => self.tima.reset(),
            0xff06 => self.tma = value,
            0xff07 => {
                self.tac = value;
                self.set_tima();
            }
            _ => panic!("invalid address {:04x} Timer", address),
        }
    }
}

struct Counter {
    frequency: u32,
    clocks: u32,
    counter: u8,
}

impl Counter {
    fn new(frequency: u32) -> Self {
        Self {
            frequency,
            clocks: 0,
            counter: 0,
        }
    }

    fn counter(&self) -> u8 {
        self.counter
    }

    fn set_counter(&mut self, c: u8) {
        self.counter = c;
    }

    fn reset(&mut self) {
        self.counter = 0;
    }

    fn tick(&mut self) -> bool {
        self.clocks += 1;
        if self.clocks == 4_194_304 / self.frequency {
            self.clocks = 0;
            self.counter = self.counter.wrapping_add(1);
            true
        } else {
            false
        }
    }
}
