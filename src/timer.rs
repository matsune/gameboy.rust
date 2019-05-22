use crate::memory::Memory;
use crate::mmu::{InterruptFlag, InterruptType};
use crate::util::is_bit_on;

#[derive(Default)]
pub struct Timer {
    div_clocks: usize,
    main_clocks: usize,
    divider: u8, // DIV
    counter: u8, // TIMA
    modulo: u8,  // TMA
    tac: TAC,
}

struct TAC {
    inner: u8,
}

impl Default for TAC {
    fn default() -> Self {
        Self::from(0)
    }
}

impl TAC {
    fn get(&self) -> u8 {
        self.inner
    }

    fn thresh(&self) -> usize {
        match self.inner & 0b11 {
            0 => 1024, // 4KHz
            1 => 16,   // 256KHz
            2 => 64,   // 64KHz
            3 => 256,  // 16KHz
            _ => panic!(),
        }
    }

    fn enable(&self) -> bool {
        is_bit_on(self.inner, 2)
    }
}

impl std::convert::From<u8> for TAC {
    fn from(inner: u8) -> Self {
        Self { inner }
    }
}

impl Timer {
    pub fn tick(&mut self, clocks: usize, int_flag: &mut InterruptFlag) {
        self.update_div(clocks);
        self.update_timer(clocks, int_flag);
    }

    fn update_div(&mut self, clocks: usize) {
        self.div_clocks += clocks;
        if self.div_clocks >= 256 {
            self.div_clocks -= 256;
            self.divider = self.divider.wrapping_add(1);
        }
    }

    fn update_timer(&mut self, clocks: usize, int_flag: &mut InterruptFlag) {
        if !self.tac.enable() {
            return;
        }
        self.main_clocks += clocks;

        let thresh = self.tac.thresh();
        while self.main_clocks >= thresh {
            self.counter = self.counter.wrapping_add(1);
            if self.counter == 0 {
                self.counter = self.modulo;
                int_flag.interrupt(InterruptType::Timer);
            }
            self.main_clocks -= thresh;
        }
    }
}

impl Memory for Timer {
    fn read(&self, a: u16) -> u8 {
        match a {
            0xff04 => self.divider,
            0xff05 => self.counter,
            0xff06 => self.modulo,
            0xff07 => self.tac.get(),
            _ => 0,
        }
    }
    fn write(&mut self, a: u16, v: u8) {
        match a {
            0xff04 => self.divider = 0,
            0xff05 => self.counter = v,
            0xff06 => self.modulo = v,
            0xff07 => self.tac = v.into(),
            _ => {}
        }
    }
}
