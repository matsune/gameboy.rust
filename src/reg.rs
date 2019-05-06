use crate::util::{get_lsb, get_msb, is_bit_on, set_bit};

#[derive(Default)]
pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub ime: bool,
}

impl std::fmt::Debug for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "AF:{:04x} BC:{:04x} DE:{:04x} HL:{:04x} SP:{:04x} PC:{:04x} flag:{}{}{}{}",
            self.af(),
            self.bc(),
            self.de(),
            self.hl(),
            self.sp,
            self.pc,
            if self.get_flag(Flag::Z) { "Z" } else { "-" },
            if self.get_flag(Flag::N) { "N" } else { "-" },
            if self.get_flag(Flag::H) { "H" } else { "-" },
            if self.get_flag(Flag::C) { "C" } else { "-" },
        )
    }
}

impl Registers {
    pub fn af(&self) -> u16 {
        u16::from(self.a) << 8 | u16::from(self.f)
    }

    pub fn bc(&self) -> u16 {
        u16::from(self.b) << 8 | u16::from(self.c)
    }

    pub fn de(&self) -> u16 {
        u16::from(self.d) << 8 | u16::from(self.e)
    }

    pub fn hl(&self) -> u16 {
        u16::from(self.h) << 8 | u16::from(self.l)
    }

    pub fn set_af(&mut self, af: u16) {
        self.a = get_msb(af);
        self.f = (af & 0xf0) as u8;
    }

    pub fn set_bc(&mut self, bc: u16) {
        self.b = get_msb(bc);
        self.c = get_lsb(bc);
    }

    pub fn set_de(&mut self, de: u16) {
        self.d = get_msb(de);
        self.e = get_lsb(de);
    }

    pub fn set_hl(&mut self, hl: u16) {
        self.h = get_msb(hl);
        self.l = get_lsb(hl);
    }

    pub fn decrement_hl(&mut self) -> u16 {
        let hl = self.hl();
        self.set_hl(hl.wrapping_sub(1));
        hl
    }

    pub fn increment_hl(&mut self) -> u16 {
        let hl = self.hl();
        self.set_hl((hl + 1) % 0xffff);
        hl
    }

    pub fn decrement_sp(&mut self) {
        self.sp = (self.sp - 1) % 0xffff;
    }

    pub fn increment_sp(&mut self) {
        self.sp = (self.sp + 1) % 0xffff;
    }
}

#[derive(Debug)]
pub enum Flag {
    // Zero Flag
    Z = 7,
    // BCD Flags
    N = 6,
    H = 5,
    // Carry Flag
    C = 4,
}

impl Registers {
    pub fn get_flag(&self, f: Flag) -> bool {
        is_bit_on(self.f, f as u8)
    }

    pub fn set_flag(&mut self, f: Flag, b: bool) {
        self.f = set_bit(self.f, f as u8, b);
    }
}
