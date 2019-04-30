fn get_msb(n: u16) -> u8 {
    (n >> 8) as u8
}

fn get_lsb(n: u16) -> u8 {
    (n & 0xff) as u8
}

fn get_bit(n: u8, pos: u8) -> bool {
    n & (1 << pos) != 0
}

fn set_bit(n: u8, pos: u8, b: bool) -> u8 {
    if b {
        n | (1 << pos)
    } else {
        !(1 << pos) & n & 0xff
    }
}

#[derive(Default)]
pub struct Registers {
    // Accumulator & Flags
    a: u8,
    f: u8,
    // BC
    b: u8,
    c: u8,
    // DE
    d: u8,
    e: u8,
    // HL
    h: u8,
    l: u8,
    // Stack Pointer
    sp: u16,
    // Program Counter
    pc: u16,
    // Interrupt Master Enable flag
    ime: bool,
}

impl std::fmt::Debug for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "AF:0x{:x} BC:0x{:x} DE:0x{:x} HL:0x{:x} SP:0x{:x} PC:0x{:x} flag:{}{}{}{}",
            self.get_af(),
            self.get_bc(),
            self.get_de(),
            self.get_hl(),
            self.get_sp(),
            self.get_pc(),
            if self.get_flag(Flag::Z) { "Z" } else { "-" },
            if self.get_flag(Flag::N) { "N" } else { "-" },
            if self.get_flag(Flag::H) { "H" } else { "-" },
            if self.get_flag(Flag::C) { "C" } else { "-" },
        )
    }
}

impl Registers {
    pub fn get_a(&self) -> u8 {
        self.a
    }

    pub fn get_f(&self) -> u8 {
        self.f
    }

    pub fn get_b(&self) -> u8 {
        self.b
    }

    pub fn get_c(&self) -> u8 {
        self.c
    }

    pub fn get_d(&self) -> u8 {
        self.d
    }

    pub fn get_e(&self) -> u8 {
        self.e
    }

    pub fn get_h(&self) -> u8 {
        self.h
    }

    pub fn get_l(&self) -> u8 {
        self.l
    }

    pub fn get_af(&self) -> u16 {
        u16::from(self.a) << 8 | u16::from(self.f)
    }

    pub fn get_bc(&self) -> u16 {
        u16::from(self.b) << 8 | u16::from(self.c)
    }

    pub fn get_de(&self) -> u16 {
        u16::from(self.d) << 8 | u16::from(self.e)
    }

    pub fn get_hl(&self) -> u16 {
        u16::from(self.h) << 8 | u16::from(self.l)
    }

    pub fn get_sp(&self) -> u16 {
        self.sp
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn set_a(&mut self, a: u8) {
        self.a = a;
    }

    pub fn set_f(&mut self, f: u8) {
        self.f = f;
    }

    pub fn set_b(&mut self, b: u8) {
        self.b = b;
    }

    pub fn set_c(&mut self, c: u8) {
        self.c = c;
    }

    pub fn set_d(&mut self, d: u8) {
        self.d = d;
    }

    pub fn set_e(&mut self, e: u8) {
        self.e = e;
    }

    pub fn set_h(&mut self, h: u8) {
        self.h = h;
    }

    pub fn set_l(&mut self, l: u8) {
        self.l = l;
    }

    pub fn set_af(&mut self, af: u16) {
        self.a = get_msb(af);
        self.f = get_lsb(af);
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

    pub fn set_sp(&mut self, sp: u16) {
        self.sp = sp;
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.pc = pc;
    }

    pub fn is_ime(&self) -> bool {
        self.ime
    }

    pub fn set_ime(&mut self, enable: bool) {
        self.ime = enable;
    }
}

#[derive(Debug)]
enum Flag {
    // Zero Flag
    Z = 7,
    // BCD Flags
    N = 6,
    H = 5,
    // Carry Flag
    C = 4,
}

impl Registers {
    fn get_flag(&self, f: Flag) -> bool {
        get_bit(self.f, f as u8)
    }

    fn set_flag(&mut self, f: Flag, b: bool) {
        self.f = set_bit(self.f, f as u8, b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_msb_lsb() {
        assert_eq!(get_msb(0b_11010010_00101101), 0b_11010010);
        assert_eq!(get_lsb(0b_11010010_00101101), 0b_00101101);
    }

    #[test]
    fn test_get_bit() {
        assert_eq!(get_bit(0b_1000_0000, 7), true);
        assert_eq!(get_bit(0b_1000_0000, 6), false);
        assert_eq!(get_bit(0b_0010_0000, 5), true);
    }

    #[test]
    fn test_set_bit() {
        assert_eq!(set_bit(0b_1000_0000, 7, false), 0b_0000_0000);
        assert_eq!(set_bit(0b_1000_0000, 6, true), 0b_1100_0000);
        assert_eq!(set_bit(0b_0000_0000, 1, true), 0b_0000_0010);
    }
}
