use crate::memory::Memory;
use crate::reg::Flag::{C, H, N, Z};
use crate::reg::Registers;

pub struct CPU {
    pub reg: Registers,
    halted: bool,
    di: u8,
    ei: u8,
    ime: bool,
}

impl CPU {
    pub fn new(skip_boot: bool, is_gbc: bool) -> Self {
        let reg = if skip_boot {
            Registers::bootstrap(is_gbc)
        } else {
            Registers::default()
        };
        CPU {
            reg,
            halted: false,
            di: 0,
            ei: 0,
            ime: false,
        }
    }
}

const OP_CYCLES: [u32; 256] = [
    1, 3, 2, 2, 1, 1, 2, 1, 5, 2, 2, 2, 1, 1, 2, 1, 0, 3, 2, 2, 1, 1, 2, 1, 3, 2, 2, 2, 1, 1, 2, 1,
    2, 3, 2, 2, 1, 1, 2, 1, 2, 2, 2, 2, 1, 1, 2, 1, 2, 3, 2, 2, 3, 3, 3, 1, 2, 2, 2, 2, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 2, 2, 2, 2, 2, 2, 0, 2, 1, 1, 1, 1, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1,
    2, 3, 3, 4, 3, 4, 2, 4, 2, 4, 3, 0, 3, 6, 2, 4, 2, 3, 3, 0, 3, 4, 2, 4, 2, 4, 3, 0, 3, 0, 2, 4,
    3, 3, 2, 0, 0, 4, 2, 4, 4, 1, 4, 0, 0, 0, 2, 4, 3, 3, 2, 1, 0, 4, 2, 4, 3, 2, 4, 1, 0, 0, 2, 4,
];

const CB_CYCLES: [u32; 256] = [
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2,
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2,
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2,
];

impl CPU {
    pub fn tick(&mut self, mem: &mut dyn Memory) -> u32 {
        self.update_ime();

        let c = self.handle_interrupts(mem);
        if c != 0 {
            return c;
        }
        if self.halted {
            return 1;
        }
        self.command(mem)
    }

    fn update_ime(&mut self) {
        match self.di {
            2 => self.di = 1,
            1 => {
                self.di = 0;
                self.ime = false;
            }
            _ => {}
        }
        match self.ei {
            2 => self.ei = 1,
            1 => {
                self.ei = 0;
                self.ime = true;
            }
            _ => {}
        }
    }

    fn handle_interrupts(&mut self, mem: &mut dyn Memory) -> u32 {
        if !self.ime && !self.halted {
            return 0;
        }
        let int_f = mem.read(0xff0f);
        let int_e = mem.read(0xffff);
        let fired = int_f & int_e;
        if fired == 0x00 {
            return 0;
        }
        self.halted = false;
        if !self.ime {
            return 0;
        }
        self.ime = false;
        let n = fired.trailing_zeros();
        let int_f = int_f & !(1 << n);
        mem.write(0xff0f, int_f);
        self.push(mem, self.reg.pc);
        self.reg.pc = 0x0040 | ((n as u16) << 3);
        4
    }

    fn command(&mut self, mem: &mut dyn Memory) -> u32 {
        let opcode = self.read_byte(mem);
        let mut cbcode: u8 = 0;
        let mut internal_delay = 0u32;
        match opcode {
            0x00 => {}
            0x01 => {
                let v = self.read_word(mem);
                self.reg.set_bc(v);
            }
            0x02 => mem.write(self.reg.bc(), self.reg.a),
            0x03 => {
                let v = self.reg.bc().wrapping_add(1);
                self.reg.set_bc(v);
            }
            0x04 => self.reg.b = self.alu_inc(self.reg.b),
            0x05 => self.reg.b = self.alu_dec(self.reg.b),
            0x06 => self.reg.b = self.read_byte(mem),
            0x07 => {
                self.reg.a = self.alu_rlc(self.reg.a);
                self.reg.set_flag(Z, false);
            }
            0x08 => {
                let a = self.read_word(mem);
                mem.write_word(a, self.reg.sp);
            }
            0x09 => self.alu_add_hl(self.reg.bc()),
            0x0a => self.reg.a = mem.read(self.reg.bc()),
            0x0b => {
                let v = self.reg.bc().wrapping_sub(1);
                self.reg.set_bc(v);
            }
            0x0c => self.reg.c = self.alu_inc(self.reg.c),
            0x0d => self.reg.c = self.alu_dec(self.reg.c),
            0x0e => self.reg.c = self.read_byte(mem),
            0x0f => {
                self.reg.a = self.alu_rrc(self.reg.a);
                self.reg.set_flag(Z, false);
            }
            0x10 => {}
            0x11 => {
                let v = self.read_word(mem);
                self.reg.set_de(v);
            }
            0x12 => mem.write(self.reg.de(), self.reg.a),
            0x13 => {
                let v = self.reg.de().wrapping_add(1);
                self.reg.set_de(v);
            }
            0x14 => self.reg.d = self.alu_inc(self.reg.d),
            0x15 => self.reg.d = self.alu_dec(self.reg.d),
            0x16 => self.reg.d = self.read_byte(mem),
            0x17 => {
                self.reg.a = self.alu_rl(self.reg.a);
                self.reg.set_flag(Z, false);
            }
            0x18 => self.alu_jr(mem),
            0x19 => self.alu_add_hl(self.reg.de()),
            0x1a => self.reg.a = mem.read(self.reg.de()),
            0x1b => {
                let v = self.reg.de().wrapping_sub(1);
                self.reg.set_de(v);
            }
            0x1c => self.reg.e = self.alu_inc(self.reg.e),
            0x1d => self.reg.e = self.alu_dec(self.reg.e),
            0x1e => self.reg.e = self.read_byte(mem),
            0x1f => {
                self.reg.a = self.alu_rr(self.reg.a);
                self.reg.set_flag(Z, false);
            }
            0x20 => {
                if !self.reg.get_flag(Z) {
                    self.alu_jr(mem);
                    internal_delay += 1;
                } else {
                    self.reg.pc += 1;
                }
            }
            0x21 => {
                let v = self.read_word(mem);
                self.reg.set_hl(v);
            }
            0x22 => {
                let a = self.reg.hl();
                mem.write(a, self.reg.a);
                self.reg.set_hl(a.wrapping_add(1));
            }
            0x23 => {
                let v = self.reg.hl().wrapping_add(1);
                self.reg.set_hl(v);
            }
            0x24 => self.reg.h = self.alu_inc(self.reg.h),
            0x25 => self.reg.h = self.alu_dec(self.reg.h),
            0x26 => self.reg.h = self.read_byte(mem),
            0x27 => self.alu_daa(),
            0x28 => {
                if self.reg.get_flag(Z) {
                    self.alu_jr(mem);
                    internal_delay += 1;
                } else {
                    self.reg.pc += 1;
                }
            }
            0x29 => self.alu_add_hl(self.reg.hl()),
            0x2a => {
                let v = self.reg.hl();
                self.reg.a = mem.read(v);
                self.reg.set_hl(v + 1);
            }
            0x2b => {
                let v = self.reg.hl().wrapping_sub(1);
                self.reg.set_hl(v);
            }
            0x2c => self.reg.l = self.alu_inc(self.reg.l),
            0x2d => self.reg.l = self.alu_dec(self.reg.l),
            0x2e => self.reg.l = self.read_byte(mem),
            0x2f => self.alu_cpl(),
            0x30 => {
                if !self.reg.get_flag(C) {
                    self.alu_jr(mem);
                    internal_delay += 1;
                } else {
                    self.reg.pc += 1;
                }
            }
            0x31 => self.reg.sp = self.read_word(mem),
            0x32 => {
                let a = self.reg.hl();
                mem.write(a, self.reg.a);
                self.reg.set_hl(a - 1);
            }
            0x33 => {
                let v = self.reg.sp.wrapping_add(1);
                self.reg.sp = v;
            }
            0x34 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_inc(v));
            }
            0x35 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_dec(v));
            }
            0x36 => {
                let a = self.reg.hl();
                let v = self.read_byte(mem);
                mem.write(a, v);
            }
            0x37 => self.alu_scf(),
            0x38 => {
                if self.reg.get_flag(C) {
                    self.alu_jr(mem);
                    internal_delay += 1;
                } else {
                    self.reg.pc += 1;
                }
            }
            0x39 => self.alu_add_hl(self.reg.sp),
            0x3a => {
                let v = self.reg.hl();
                self.reg.a = mem.read(v);
                self.reg.set_hl(v - 1);
            }
            0x3b => {
                let v = self.reg.sp.wrapping_sub(1);
                self.reg.sp = v;
            }
            0x3c => self.reg.a = self.alu_inc(self.reg.a),
            0x3d => self.reg.a = self.alu_dec(self.reg.a),
            0x3e => self.reg.a = self.read_byte(mem),
            0x3f => self.alu_ccf(),
            0x40 => {}
            0x41 => self.reg.b = self.reg.c,
            0x42 => self.reg.b = self.reg.d,
            0x43 => self.reg.b = self.reg.e,
            0x44 => self.reg.b = self.reg.h,
            0x45 => self.reg.b = self.reg.l,
            0x46 => self.reg.b = mem.read(self.reg.hl()),
            0x47 => self.reg.b = self.reg.a,
            0x48 => self.reg.c = self.reg.b,
            0x49 => {}
            0x4a => self.reg.c = self.reg.d,
            0x4b => self.reg.c = self.reg.e,
            0x4c => self.reg.c = self.reg.h,
            0x4d => self.reg.c = self.reg.l,
            0x4e => self.reg.c = mem.read(self.reg.hl()),
            0x4f => self.reg.c = self.reg.a,
            0x50 => self.reg.d = self.reg.b,
            0x51 => self.reg.d = self.reg.c,
            0x52 => {}
            0x53 => self.reg.d = self.reg.e,
            0x54 => self.reg.d = self.reg.h,
            0x55 => self.reg.d = self.reg.l,
            0x56 => self.reg.d = mem.read(self.reg.hl()),
            0x57 => self.reg.d = self.reg.a,
            0x58 => self.reg.e = self.reg.b,
            0x59 => self.reg.e = self.reg.c,
            0x5a => self.reg.e = self.reg.d,
            0x5b => {}
            0x5c => self.reg.e = self.reg.h,
            0x5d => self.reg.e = self.reg.l,
            0x5e => self.reg.e = mem.read(self.reg.hl()),
            0x5f => self.reg.e = self.reg.a,
            0x60 => self.reg.h = self.reg.b,
            0x61 => self.reg.h = self.reg.c,
            0x62 => self.reg.h = self.reg.d,
            0x63 => self.reg.h = self.reg.e,
            0x64 => {}
            0x65 => self.reg.h = self.reg.l,
            0x66 => self.reg.h = mem.read(self.reg.hl()),
            0x67 => self.reg.h = self.reg.a,
            0x68 => self.reg.l = self.reg.b,
            0x69 => self.reg.l = self.reg.c,
            0x6a => self.reg.l = self.reg.d,
            0x6b => self.reg.l = self.reg.e,
            0x6c => self.reg.l = self.reg.h,
            0x6d => {}
            0x6e => self.reg.l = mem.read(self.reg.hl()),
            0x6f => self.reg.l = self.reg.a,
            0x70 => mem.write(self.reg.hl(), self.reg.b),
            0x71 => mem.write(self.reg.hl(), self.reg.c),
            0x72 => mem.write(self.reg.hl(), self.reg.d),
            0x73 => mem.write(self.reg.hl(), self.reg.e),
            0x74 => mem.write(self.reg.hl(), self.reg.h),
            0x75 => mem.write(self.reg.hl(), self.reg.l),
            0x76 => self.halted = true,
            0x77 => mem.write(self.reg.hl(), self.reg.a),
            0x78 => self.reg.a = self.reg.b,
            0x79 => self.reg.a = self.reg.c,
            0x7a => self.reg.a = self.reg.d,
            0x7b => self.reg.a = self.reg.e,
            0x7c => self.reg.a = self.reg.h,
            0x7d => self.reg.a = self.reg.l,
            0x7e => self.reg.a = mem.read(self.reg.hl()),
            0x7f => {}
            0x80 => self.alu_add(self.reg.b),
            0x81 => self.alu_add(self.reg.c),
            0x82 => self.alu_add(self.reg.d),
            0x83 => self.alu_add(self.reg.e),
            0x84 => self.alu_add(self.reg.h),
            0x85 => self.alu_add(self.reg.l),
            0x86 => self.alu_add(mem.read(self.reg.hl())),
            0x87 => self.alu_add(self.reg.a),
            0x88 => self.alu_adc(self.reg.b),
            0x89 => self.alu_adc(self.reg.c),
            0x8a => self.alu_adc(self.reg.d),
            0x8b => self.alu_adc(self.reg.e),
            0x8c => self.alu_adc(self.reg.h),
            0x8d => self.alu_adc(self.reg.l),
            0x8e => self.alu_adc(mem.read(self.reg.hl())),
            0x8f => self.alu_adc(self.reg.a),
            0x90 => self.alu_sub(self.reg.b),
            0x91 => self.alu_sub(self.reg.c),
            0x92 => self.alu_sub(self.reg.d),
            0x93 => self.alu_sub(self.reg.e),
            0x94 => self.alu_sub(self.reg.h),
            0x95 => self.alu_sub(self.reg.l),
            0x96 => self.alu_sub(mem.read(self.reg.hl())),
            0x97 => self.alu_sub(self.reg.a),
            0x98 => self.alu_sbc(self.reg.b),
            0x99 => self.alu_sbc(self.reg.c),
            0x9a => self.alu_sbc(self.reg.d),
            0x9b => self.alu_sbc(self.reg.e),
            0x9c => self.alu_sbc(self.reg.h),
            0x9d => self.alu_sbc(self.reg.l),
            0x9e => self.alu_sbc(mem.read(self.reg.hl())),
            0x9f => self.alu_sbc(self.reg.a),
            0xa0 => self.alu_and(self.reg.b),
            0xa1 => self.alu_and(self.reg.c),
            0xa2 => self.alu_and(self.reg.d),
            0xa3 => self.alu_and(self.reg.e),
            0xa4 => self.alu_and(self.reg.h),
            0xa5 => self.alu_and(self.reg.l),
            0xa6 => self.alu_and(mem.read(self.reg.hl())),
            0xa7 => self.alu_and(self.reg.a),
            0xa8 => self.alu_xor(self.reg.b),
            0xa9 => self.alu_xor(self.reg.c),
            0xaa => self.alu_xor(self.reg.d),
            0xab => self.alu_xor(self.reg.e),
            0xac => self.alu_xor(self.reg.h),
            0xad => self.alu_xor(self.reg.l),
            0xae => self.alu_xor(mem.read(self.reg.hl())),
            0xaf => self.alu_xor(self.reg.a),
            0xb0 => self.alu_or(self.reg.b),
            0xb1 => self.alu_or(self.reg.c),
            0xb2 => self.alu_or(self.reg.d),
            0xb3 => self.alu_or(self.reg.e),
            0xb4 => self.alu_or(self.reg.h),
            0xb5 => self.alu_or(self.reg.l),
            0xb6 => self.alu_or(mem.read(self.reg.hl())),
            0xb7 => self.alu_or(self.reg.a),
            0xb8 => self.alu_cp(self.reg.b),
            0xb9 => self.alu_cp(self.reg.c),
            0xba => self.alu_cp(self.reg.d),
            0xbb => self.alu_cp(self.reg.e),
            0xbc => self.alu_cp(self.reg.h),
            0xbd => self.alu_cp(self.reg.l),
            0xbe => self.alu_cp(mem.read(self.reg.hl())),
            0xbf => self.alu_cp(self.reg.a),
            0xc0 => {
                if !self.reg.get_flag(Z) {
                    self.reg.pc = self.pop(mem);
                    internal_delay += 3;
                }
            }
            0xc1 => {
                let v = self.pop(mem);
                self.reg.set_bc(v);
            }
            0xc2 => {
                let pc = self.read_word(mem);
                if !self.reg.get_flag(Z) {
                    self.reg.pc = pc;
                    internal_delay += 1;
                }
            }
            0xc3 => self.reg.pc = mem.read_word(self.reg.pc),
            0xc4 => {
                if !self.reg.get_flag(Z) {
                    self.push(mem, self.reg.pc + 2);
                    self.reg.pc = mem.read_word(self.reg.pc);
                    internal_delay += 3;
                } else {
                    self.reg.pc += 2;
                }
            }
            0xc5 => self.push(mem, self.reg.bc()),
            0xc6 => {
                let v = self.read_byte(mem);
                self.alu_add(v);
            }
            0xc7 => {
                self.push(mem, self.reg.pc);
                self.reg.pc = 0x00;
            }
            0xc8 => {
                if self.reg.get_flag(Z) {
                    self.reg.pc = self.pop(mem);
                    internal_delay += 3;
                }
            }
            0xc9 => {
                self.reg.pc = self.pop(mem);
            }
            0xca => {
                let pc = self.read_word(mem);
                if self.reg.get_flag(Z) {
                    self.reg.pc = pc;
                    internal_delay += 1;
                }
            }
            0xcb => {
                cbcode = mem.read(self.reg.pc);
                self.reg.pc += 1;
                self.ext_command(mem, cbcode);
            }
            0xcc => {
                if self.reg.get_flag(Z) {
                    self.push(mem, self.reg.pc + 2);
                    self.reg.pc = mem.read_word(self.reg.pc);
                    internal_delay += 3;
                } else {
                    self.reg.pc += 2;
                }
            }
            0xcd => {
                self.push(mem, self.reg.pc + 2);
                self.reg.pc = mem.read_word(self.reg.pc);
            }
            0xce => {
                let v = self.read_byte(mem);
                self.alu_adc(v);
            }
            0xcf => {
                self.push(mem, self.reg.pc);
                self.reg.pc = 0x08;
            }
            0xd0 => {
                if !self.reg.get_flag(C) {
                    self.reg.pc = self.pop(mem);
                    internal_delay += 3;
                }
            }
            0xd1 => {
                let v = self.pop(mem);
                self.reg.set_de(v);
            }
            0xd2 => {
                let pc = self.read_word(mem);
                if !self.reg.get_flag(C) {
                    self.reg.pc = pc;
                    internal_delay += 1;
                }
            }
            0xd4 => {
                if !self.reg.get_flag(C) {
                    self.push(mem, self.reg.pc + 2);
                    self.reg.pc = mem.read_word(self.reg.pc);
                    internal_delay += 3;
                } else {
                    self.reg.pc += 2;
                }
            }
            0xd5 => self.push(mem, self.reg.de()),
            0xd6 => {
                let v = self.read_byte(mem);
                self.alu_sub(v);
            }
            0xd7 => {
                self.push(mem, self.reg.pc);
                self.reg.pc = 0x10;
            }
            0xd8 => {
                if self.reg.get_flag(C) {
                    self.reg.pc = self.pop(mem);
                    internal_delay += 3;
                }
            }
            0xd9 => {
                self.reg.pc = self.pop(mem);
                self.ime = true;
            }
            0xda => {
                let pc = self.read_word(mem);
                if self.reg.get_flag(C) {
                    self.reg.pc = pc;
                    internal_delay += 1;
                }
            }
            0xdc => {
                if self.reg.get_flag(C) {
                    self.push(mem, self.reg.pc + 2);
                    self.reg.pc = mem.read_word(self.reg.pc);
                    internal_delay += 3;
                } else {
                    self.reg.pc += 2;
                }
            }
            0xde => {
                let v = self.read_byte(mem);
                self.alu_sbc(v);
            }
            0xdf => {
                self.push(mem, self.reg.pc);
                self.reg.pc = 0x18;
            }
            0xe0 => {
                let a = 0xff00 | u16::from(self.read_byte(mem));
                mem.write(a, self.reg.a);
            }
            0xe1 => {
                let v = self.pop(mem);
                self.reg.set_hl(v);
            }
            0xe2 => mem.write(0xff00 | u16::from(self.reg.c), self.reg.a),
            0xe5 => self.push(mem, self.reg.hl()),
            0xe6 => {
                let v = self.read_byte(mem);
                self.alu_and(v);
            }
            0xe7 => {
                self.push(mem, self.reg.pc);
                self.reg.pc = 0x20;
            }
            0xe8 => self.alu_add_sp(mem),
            0xe9 => self.reg.pc = self.reg.hl(),
            0xea => {
                let a = self.read_word(mem);
                mem.write(a, self.reg.a);
            }
            0xee => {
                let v = self.read_byte(mem);
                self.alu_xor(v);
            }
            0xef => {
                self.push(mem, self.reg.pc);
                self.reg.pc = 0x28;
            }
            0xf0 => {
                let a = 0xff00 | u16::from(self.read_byte(mem));
                self.reg.a = mem.read(a);
            }
            0xf1 => {
                let v = self.pop(mem);
                self.reg.set_af(v);
            }
            0xf2 => self.reg.a = mem.read(0xff00 | u16::from(self.reg.c)),
            0xf3 => self.di = 2,
            0xf5 => self.push(mem, self.reg.af()),
            0xf6 => {
                let v = self.read_byte(mem);
                self.alu_or(v);
            }
            0xf7 => {
                self.push(mem, self.reg.pc);
                self.reg.pc = 0x30;
            }
            0xf8 => {
                let a = self.reg.sp;
                let b = i16::from(self.read_byte(mem) as i8) as u16;
                self.reg.set_flag(C, (a & 0x00ff) + (b & 0x00ff) > 0x00ff);
                self.reg.set_flag(H, (a & 0x000f) + (b & 0x000f) > 0x000f);
                self.reg.set_flag(N, false);
                self.reg.set_flag(Z, false);
                self.reg.set_hl(a.wrapping_add(b));
            }
            0xf9 => self.reg.sp = self.reg.hl(),
            0xfa => {
                let a = self.read_word(mem);
                self.reg.a = mem.read(a);
            }
            0xfb => self.ei = 2,
            0xfe => {
                let v = self.read_byte(mem);
                self.alu_cp(v);
            }
            0xff => {
                self.push(mem, self.reg.pc);
                self.reg.pc = 0x38;
            }
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        if opcode == 0xcb {
            CB_CYCLES[cbcode as usize]
        } else {
            OP_CYCLES[opcode as usize] + internal_delay
        }
    }

    fn ext_command(&mut self, mem: &mut dyn Memory, opcode: u8) {
        match opcode {
            0x00 => self.reg.b = self.alu_rlc(self.reg.b),
            0x01 => self.reg.c = self.alu_rlc(self.reg.c),
            0x02 => self.reg.d = self.alu_rlc(self.reg.d),
            0x03 => self.reg.e = self.alu_rlc(self.reg.e),
            0x04 => self.reg.h = self.alu_rlc(self.reg.h),
            0x05 => self.reg.l = self.alu_rlc(self.reg.l),
            0x06 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_rlc(v));
            }
            0x07 => self.reg.a = self.alu_rlc(self.reg.a),
            0x08 => self.reg.b = self.alu_rrc(self.reg.b),
            0x09 => self.reg.c = self.alu_rrc(self.reg.c),
            0x0a => self.reg.d = self.alu_rrc(self.reg.d),
            0x0b => self.reg.e = self.alu_rrc(self.reg.e),
            0x0c => self.reg.h = self.alu_rrc(self.reg.h),
            0x0d => self.reg.l = self.alu_rrc(self.reg.l),
            0x0e => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_rrc(v));
            }
            0x0f => self.reg.a = self.alu_rrc(self.reg.a),
            0x10 => self.reg.b = self.alu_rl(self.reg.b),
            0x11 => self.reg.c = self.alu_rl(self.reg.c),
            0x12 => self.reg.d = self.alu_rl(self.reg.d),
            0x13 => self.reg.e = self.alu_rl(self.reg.e),
            0x14 => self.reg.h = self.alu_rl(self.reg.h),
            0x15 => self.reg.l = self.alu_rl(self.reg.l),
            0x16 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_rl(v));
            }
            0x17 => self.reg.a = self.alu_rl(self.reg.a),
            0x18 => self.reg.b = self.alu_rr(self.reg.b),
            0x19 => self.reg.c = self.alu_rr(self.reg.c),
            0x1a => self.reg.d = self.alu_rr(self.reg.d),
            0x1b => self.reg.e = self.alu_rr(self.reg.e),
            0x1c => self.reg.h = self.alu_rr(self.reg.h),
            0x1d => self.reg.l = self.alu_rr(self.reg.l),
            0x1e => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_rr(v));
            }
            0x1f => self.reg.a = self.alu_rr(self.reg.a),
            0x20 => self.reg.b = self.alu_sla(self.reg.b),
            0x21 => self.reg.c = self.alu_sla(self.reg.c),
            0x22 => self.reg.d = self.alu_sla(self.reg.d),
            0x23 => self.reg.e = self.alu_sla(self.reg.e),
            0x24 => self.reg.h = self.alu_sla(self.reg.h),
            0x25 => self.reg.l = self.alu_sla(self.reg.l),
            0x26 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_sla(v));
            }
            0x27 => self.reg.a = self.alu_sla(self.reg.a),
            0x28 => self.reg.b = self.alu_sra(self.reg.b),
            0x29 => self.reg.c = self.alu_sra(self.reg.c),
            0x2a => self.reg.d = self.alu_sra(self.reg.d),
            0x2b => self.reg.e = self.alu_sra(self.reg.e),
            0x2c => self.reg.h = self.alu_sra(self.reg.h),
            0x2d => self.reg.l = self.alu_sra(self.reg.l),
            0x2e => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_sra(v));
            }
            0x2f => self.reg.a = self.alu_sra(self.reg.a),
            0x30 => self.reg.b = self.alu_swap(self.reg.b),
            0x31 => self.reg.c = self.alu_swap(self.reg.c),
            0x32 => self.reg.d = self.alu_swap(self.reg.d),
            0x33 => self.reg.e = self.alu_swap(self.reg.e),
            0x34 => self.reg.h = self.alu_swap(self.reg.h),
            0x35 => self.reg.l = self.alu_swap(self.reg.l),
            0x36 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_swap(v));
            }
            0x37 => self.reg.a = self.alu_swap(self.reg.a),
            0x38 => self.reg.b = self.alu_srl(self.reg.b),
            0x39 => self.reg.c = self.alu_srl(self.reg.c),
            0x3a => self.reg.d = self.alu_srl(self.reg.d),
            0x3b => self.reg.e = self.alu_srl(self.reg.e),
            0x3c => self.reg.h = self.alu_srl(self.reg.h),
            0x3d => self.reg.l = self.alu_srl(self.reg.l),
            0x3e => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_srl(v));
            }
            0x3f => self.reg.a = self.alu_srl(self.reg.a),
            0x40 => self.alu_bit(self.reg.b, 0),
            0x41 => self.alu_bit(self.reg.c, 0),
            0x42 => self.alu_bit(self.reg.d, 0),
            0x43 => self.alu_bit(self.reg.e, 0),
            0x44 => self.alu_bit(self.reg.h, 0),
            0x45 => self.alu_bit(self.reg.l, 0),
            0x46 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                self.alu_bit(v, 0);
            }
            0x47 => self.alu_bit(self.reg.a, 0),
            0x48 => self.alu_bit(self.reg.b, 1),
            0x49 => self.alu_bit(self.reg.c, 1),
            0x4a => self.alu_bit(self.reg.d, 1),
            0x4b => self.alu_bit(self.reg.e, 1),
            0x4c => self.alu_bit(self.reg.h, 1),
            0x4d => self.alu_bit(self.reg.l, 1),
            0x4e => {
                let a = self.reg.hl();
                let v = mem.read(a);
                self.alu_bit(v, 1);
            }
            0x4f => self.alu_bit(self.reg.a, 1),
            0x50 => self.alu_bit(self.reg.b, 2),
            0x51 => self.alu_bit(self.reg.c, 2),
            0x52 => self.alu_bit(self.reg.d, 2),
            0x53 => self.alu_bit(self.reg.e, 2),
            0x54 => self.alu_bit(self.reg.h, 2),
            0x55 => self.alu_bit(self.reg.l, 2),
            0x56 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                self.alu_bit(v, 2);
            }
            0x57 => self.alu_bit(self.reg.a, 2),
            0x58 => self.alu_bit(self.reg.b, 3),
            0x59 => self.alu_bit(self.reg.c, 3),
            0x5a => self.alu_bit(self.reg.d, 3),
            0x5b => self.alu_bit(self.reg.e, 3),
            0x5c => self.alu_bit(self.reg.h, 3),
            0x5d => self.alu_bit(self.reg.l, 3),
            0x5e => {
                let a = self.reg.hl();
                let v = mem.read(a);
                self.alu_bit(v, 3);
            }
            0x5f => self.alu_bit(self.reg.a, 3),
            0x60 => self.alu_bit(self.reg.b, 4),
            0x61 => self.alu_bit(self.reg.c, 4),
            0x62 => self.alu_bit(self.reg.d, 4),
            0x63 => self.alu_bit(self.reg.e, 4),
            0x64 => self.alu_bit(self.reg.h, 4),
            0x65 => self.alu_bit(self.reg.l, 4),
            0x66 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                self.alu_bit(v, 4);
            }
            0x67 => self.alu_bit(self.reg.a, 4),
            0x68 => self.alu_bit(self.reg.b, 5),
            0x69 => self.alu_bit(self.reg.c, 5),
            0x6a => self.alu_bit(self.reg.d, 5),
            0x6b => self.alu_bit(self.reg.e, 5),
            0x6c => self.alu_bit(self.reg.h, 5),
            0x6d => self.alu_bit(self.reg.l, 5),
            0x6e => {
                let a = self.reg.hl();
                let v = mem.read(a);
                self.alu_bit(v, 5);
            }
            0x6f => self.alu_bit(self.reg.a, 5),
            0x70 => self.alu_bit(self.reg.b, 6),
            0x71 => self.alu_bit(self.reg.c, 6),
            0x72 => self.alu_bit(self.reg.d, 6),
            0x73 => self.alu_bit(self.reg.e, 6),
            0x74 => self.alu_bit(self.reg.h, 6),
            0x75 => self.alu_bit(self.reg.l, 6),
            0x76 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                self.alu_bit(v, 6);
            }
            0x77 => self.alu_bit(self.reg.a, 6),
            0x78 => self.alu_bit(self.reg.b, 7),
            0x79 => self.alu_bit(self.reg.c, 7),
            0x7a => self.alu_bit(self.reg.d, 7),
            0x7b => self.alu_bit(self.reg.e, 7),
            0x7c => self.alu_bit(self.reg.h, 7),
            0x7d => self.alu_bit(self.reg.l, 7),
            0x7e => {
                let a = self.reg.hl();
                let v = mem.read(a);
                self.alu_bit(v, 7);
            }
            0x7f => self.alu_bit(self.reg.a, 7),
            0x80 => self.reg.b = self.alu_res(self.reg.b, 0),
            0x81 => self.reg.c = self.alu_res(self.reg.c, 0),
            0x82 => self.reg.d = self.alu_res(self.reg.d, 0),
            0x83 => self.reg.e = self.alu_res(self.reg.e, 0),
            0x84 => self.reg.h = self.alu_res(self.reg.h, 0),
            0x85 => self.reg.l = self.alu_res(self.reg.l, 0),
            0x86 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_res(v, 0));
            }
            0x87 => self.reg.a = self.alu_res(self.reg.a, 0),
            0x88 => self.reg.b = self.alu_res(self.reg.b, 1),
            0x89 => self.reg.c = self.alu_res(self.reg.c, 1),
            0x8a => self.reg.d = self.alu_res(self.reg.d, 1),
            0x8b => self.reg.e = self.alu_res(self.reg.e, 1),
            0x8c => self.reg.h = self.alu_res(self.reg.h, 1),
            0x8d => self.reg.l = self.alu_res(self.reg.l, 1),
            0x8e => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_res(v, 1));
            }
            0x8f => self.reg.a = self.alu_res(self.reg.a, 1),
            0x90 => self.reg.b = self.alu_res(self.reg.b, 2),
            0x91 => self.reg.c = self.alu_res(self.reg.c, 2),
            0x92 => self.reg.d = self.alu_res(self.reg.d, 2),
            0x93 => self.reg.e = self.alu_res(self.reg.e, 2),
            0x94 => self.reg.h = self.alu_res(self.reg.h, 2),
            0x95 => self.reg.l = self.alu_res(self.reg.l, 2),
            0x96 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_res(v, 2));
            }
            0x97 => self.reg.a = self.alu_res(self.reg.a, 2),
            0x98 => self.reg.b = self.alu_res(self.reg.b, 3),
            0x99 => self.reg.c = self.alu_res(self.reg.c, 3),
            0x9a => self.reg.d = self.alu_res(self.reg.d, 3),
            0x9b => self.reg.e = self.alu_res(self.reg.e, 3),
            0x9c => self.reg.h = self.alu_res(self.reg.h, 3),
            0x9d => self.reg.l = self.alu_res(self.reg.l, 3),
            0x9e => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_res(v, 3));
            }
            0x9f => self.reg.a = self.alu_res(self.reg.a, 3),
            0xa0 => self.reg.b = self.alu_res(self.reg.b, 4),
            0xa1 => self.reg.c = self.alu_res(self.reg.c, 4),
            0xa2 => self.reg.d = self.alu_res(self.reg.d, 4),
            0xa3 => self.reg.e = self.alu_res(self.reg.e, 4),
            0xa4 => self.reg.h = self.alu_res(self.reg.h, 4),
            0xa5 => self.reg.l = self.alu_res(self.reg.l, 4),
            0xa6 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_res(v, 4));
            }
            0xa7 => self.reg.a = self.alu_res(self.reg.a, 4),
            0xa8 => self.reg.b = self.alu_res(self.reg.b, 5),
            0xa9 => self.reg.c = self.alu_res(self.reg.c, 5),
            0xaa => self.reg.d = self.alu_res(self.reg.d, 5),
            0xab => self.reg.e = self.alu_res(self.reg.e, 5),
            0xac => self.reg.h = self.alu_res(self.reg.h, 5),
            0xad => self.reg.l = self.alu_res(self.reg.l, 5),
            0xae => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_res(v, 5));
            }
            0xaf => self.reg.a = self.alu_res(self.reg.a, 5),
            0xb0 => self.reg.b = self.alu_res(self.reg.b, 6),
            0xb1 => self.reg.c = self.alu_res(self.reg.c, 6),
            0xb2 => self.reg.d = self.alu_res(self.reg.d, 6),
            0xb3 => self.reg.e = self.alu_res(self.reg.e, 6),
            0xb4 => self.reg.h = self.alu_res(self.reg.h, 6),
            0xb5 => self.reg.l = self.alu_res(self.reg.l, 6),
            0xb6 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_res(v, 6));
            }
            0xb7 => self.reg.a = self.alu_res(self.reg.a, 6),
            0xb8 => self.reg.b = self.alu_res(self.reg.b, 7),
            0xb9 => self.reg.c = self.alu_res(self.reg.c, 7),
            0xba => self.reg.d = self.alu_res(self.reg.d, 7),
            0xbb => self.reg.e = self.alu_res(self.reg.e, 7),
            0xbc => self.reg.h = self.alu_res(self.reg.h, 7),
            0xbd => self.reg.l = self.alu_res(self.reg.l, 7),
            0xbe => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_res(v, 7));
            }
            0xbf => self.reg.a = self.alu_res(self.reg.a, 7),
            0xc0 => self.reg.b = self.alu_set(self.reg.b, 0),
            0xc1 => self.reg.c = self.alu_set(self.reg.c, 0),
            0xc2 => self.reg.d = self.alu_set(self.reg.d, 0),
            0xc3 => self.reg.e = self.alu_set(self.reg.e, 0),
            0xc4 => self.reg.h = self.alu_set(self.reg.h, 0),
            0xc5 => self.reg.l = self.alu_set(self.reg.l, 0),
            0xc6 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_set(v, 0));
            }
            0xc7 => self.reg.a = self.alu_set(self.reg.a, 0),
            0xc8 => self.reg.b = self.alu_set(self.reg.b, 1),
            0xc9 => self.reg.c = self.alu_set(self.reg.c, 1),
            0xca => self.reg.d = self.alu_set(self.reg.d, 1),
            0xcb => self.reg.e = self.alu_set(self.reg.e, 1),
            0xcc => self.reg.h = self.alu_set(self.reg.h, 1),
            0xcd => self.reg.l = self.alu_set(self.reg.l, 1),
            0xce => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_set(v, 1));
            }
            0xcf => self.reg.a = self.alu_set(self.reg.a, 1),
            0xd0 => self.reg.b = self.alu_set(self.reg.b, 2),
            0xd1 => self.reg.c = self.alu_set(self.reg.c, 2),
            0xd2 => self.reg.d = self.alu_set(self.reg.d, 2),
            0xd3 => self.reg.e = self.alu_set(self.reg.e, 2),
            0xd4 => self.reg.h = self.alu_set(self.reg.h, 2),
            0xd5 => self.reg.l = self.alu_set(self.reg.l, 2),
            0xd6 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_set(v, 2));
            }
            0xd7 => self.reg.a = self.alu_set(self.reg.a, 2),
            0xd8 => self.reg.b = self.alu_set(self.reg.b, 3),
            0xd9 => self.reg.c = self.alu_set(self.reg.c, 3),
            0xda => self.reg.d = self.alu_set(self.reg.d, 3),
            0xdb => self.reg.e = self.alu_set(self.reg.e, 3),
            0xdc => self.reg.h = self.alu_set(self.reg.h, 3),
            0xdd => self.reg.l = self.alu_set(self.reg.l, 3),
            0xde => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_set(v, 3));
            }
            0xdf => self.reg.a = self.alu_set(self.reg.a, 3),
            0xe0 => self.reg.b = self.alu_set(self.reg.b, 4),
            0xe1 => self.reg.c = self.alu_set(self.reg.c, 4),
            0xe2 => self.reg.d = self.alu_set(self.reg.d, 4),
            0xe3 => self.reg.e = self.alu_set(self.reg.e, 4),
            0xe4 => self.reg.h = self.alu_set(self.reg.h, 4),
            0xe5 => self.reg.l = self.alu_set(self.reg.l, 4),
            0xe6 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_set(v, 4));
            }
            0xe7 => self.reg.a = self.alu_set(self.reg.a, 4),
            0xe8 => self.reg.b = self.alu_set(self.reg.b, 5),
            0xe9 => self.reg.c = self.alu_set(self.reg.c, 5),
            0xea => self.reg.d = self.alu_set(self.reg.d, 5),
            0xeb => self.reg.e = self.alu_set(self.reg.e, 5),
            0xec => self.reg.h = self.alu_set(self.reg.h, 5),
            0xed => self.reg.l = self.alu_set(self.reg.l, 5),
            0xee => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_set(v, 5));
            }
            0xef => self.reg.a = self.alu_set(self.reg.a, 5),
            0xf0 => self.reg.b = self.alu_set(self.reg.b, 6),
            0xf1 => self.reg.c = self.alu_set(self.reg.c, 6),
            0xf2 => self.reg.d = self.alu_set(self.reg.d, 6),
            0xf3 => self.reg.e = self.alu_set(self.reg.e, 6),
            0xf4 => self.reg.h = self.alu_set(self.reg.h, 6),
            0xf5 => self.reg.l = self.alu_set(self.reg.l, 6),
            0xf6 => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_set(v, 6));
            }
            0xf7 => self.reg.a = self.alu_set(self.reg.a, 6),
            0xf8 => self.reg.b = self.alu_set(self.reg.b, 7),
            0xf9 => self.reg.c = self.alu_set(self.reg.c, 7),
            0xfa => self.reg.d = self.alu_set(self.reg.d, 7),
            0xfb => self.reg.e = self.alu_set(self.reg.e, 7),
            0xfc => self.reg.h = self.alu_set(self.reg.h, 7),
            0xfd => self.reg.l = self.alu_set(self.reg.l, 7),
            0xfe => {
                let a = self.reg.hl();
                let v = mem.read(a);
                mem.write(a, self.alu_set(v, 7));
            }
            0xff => self.reg.a = self.alu_set(self.reg.a, 7),
        }
    }
}

// alu
impl CPU {
    fn read_byte(&mut self, mem: &mut dyn Memory) -> u8 {
        let v = mem.read(self.reg.pc);
        self.reg.pc += 1;
        v
    }

    fn read_word(&mut self, mem: &mut dyn Memory) -> u16 {
        let v = mem.read_word(self.reg.pc);
        self.reg.pc += 2;
        v
    }

    fn push(&mut self, mem: &mut dyn Memory, v: u16) {
        self.reg.sp -= 2;
        mem.write_word(self.reg.sp, v);
    }

    fn pop(&mut self, mem: &mut dyn Memory) -> u16 {
        let r = mem.read_word(self.reg.sp);
        self.reg.sp += 2;
        r
    }

    fn alu_add(&mut self, n: u8) {
        let a = self.reg.a;
        let r = a.wrapping_add(n);
        self.reg.set_flag(C, u16::from(a) + u16::from(n) > 0xff);
        self.reg.set_flag(H, (a & 0x0f) + (n & 0x0f) > 0x0f);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, r == 0x00);
        self.reg.a = r;
    }

    fn alu_adc(&mut self, n: u8) {
        let a = self.reg.a;
        let c = u8::from(self.reg.get_flag(C));
        let r = a.wrapping_add(n).wrapping_add(c);
        self.reg
            .set_flag(C, u16::from(a) + u16::from(n) + u16::from(c) > 0xff);
        self.reg
            .set_flag(H, (a & 0x0f) + (n & 0x0f) + (c & 0x0f) > 0x0f);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, r == 0x00);
        self.reg.a = r;
    }

    fn alu_sub(&mut self, n: u8) {
        let a = self.reg.a;
        let r = a.wrapping_sub(n);
        self.reg.set_flag(C, u16::from(a) < u16::from(n));
        self.reg.set_flag(H, (a & 0x0f) < (n & 0x0f));
        self.reg.set_flag(N, true);
        self.reg.set_flag(Z, r == 0x00);
        self.reg.a = r;
    }

    fn alu_sbc(&mut self, n: u8) {
        let a = self.reg.a;
        let c = u8::from(self.reg.get_flag(C));
        let r = a.wrapping_sub(n).wrapping_sub(c);
        self.reg
            .set_flag(C, u16::from(a) < u16::from(n) + u16::from(c));
        self.reg.set_flag(H, (a & 0x0f) < (n & 0x0f) + c);
        self.reg.set_flag(N, true);
        self.reg.set_flag(Z, r == 0x00);
        self.reg.a = r;
    }

    fn alu_and(&mut self, n: u8) {
        let r = self.reg.a & n;
        self.reg.set_flag(C, false);
        self.reg.set_flag(H, true);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, r == 0x00);
        self.reg.a = r;
    }

    fn alu_or(&mut self, n: u8) {
        let r = self.reg.a | n;
        self.reg.set_flag(C, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, r == 0x00);
        self.reg.a = r;
    }

    fn alu_xor(&mut self, n: u8) {
        let r = self.reg.a ^ n;
        self.reg.set_flag(C, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, r == 0x00);
        self.reg.a = r;
    }

    fn alu_cp(&mut self, n: u8) {
        let r = self.reg.a;
        self.alu_sub(n);
        self.reg.a = r;
    }

    fn alu_inc(&mut self, a: u8) -> u8 {
        let r = a.wrapping_add(1);
        self.reg.set_flag(H, (a & 0x0f) + 0x01 > 0x0f);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, r == 0x00);
        r
    }

    fn alu_dec(&mut self, a: u8) -> u8 {
        let r = a.wrapping_sub(1);
        self.reg.set_flag(H, a.trailing_zeros() >= 4);
        self.reg.set_flag(N, true);
        self.reg.set_flag(Z, r == 0);
        r
    }

    fn alu_add_hl(&mut self, n: u16) {
        let a = self.reg.hl();
        let r = a.wrapping_add(n);
        self.reg.set_flag(C, a > 0xffff - n);
        self.reg.set_flag(H, (a & 0x07ff) + (n & 0x07ff) > 0x07ff);
        self.reg.set_flag(N, false);
        self.reg.set_hl(r);
    }

    fn alu_add_sp(&mut self, mem: &mut dyn Memory) {
        let a = self.reg.sp;
        let b = i16::from(self.read_byte(mem) as i8) as u16;
        self.reg.set_flag(C, (a & 0x00ff) + (b & 0x00ff) > 0x00ff);
        self.reg.set_flag(H, (a & 0x000f) + (b & 0x000f) > 0x000f);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, false);
        self.reg.sp = a.wrapping_add(b);
    }

    fn alu_swap(&mut self, a: u8) -> u8 {
        self.reg.set_flag(C, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, a == 0x00);
        (a >> 4) | (a << 4)
    }

    fn alu_daa(&mut self) {
        let mut a = self.reg.a;
        let mut adjust = if self.reg.get_flag(C) { 0x60 } else { 0x00 };
        if self.reg.get_flag(H) {
            adjust |= 0x06;
        };
        if !self.reg.get_flag(N) {
            if a & 0x0f > 0x09 {
                adjust |= 0x06;
            };
            if a > 0x99 {
                adjust |= 0x60;
            };
            a = a.wrapping_add(adjust);
        } else {
            a = a.wrapping_sub(adjust);
        }
        self.reg.set_flag(C, adjust >= 0x60);
        self.reg.set_flag(H, false);
        self.reg.set_flag(Z, a == 0x00);
        self.reg.a = a;
    }

    fn alu_cpl(&mut self) {
        self.reg.a = !self.reg.a;
        self.reg.set_flag(H, true);
        self.reg.set_flag(N, true);
    }

    fn alu_ccf(&mut self) {
        let v = !self.reg.get_flag(C);
        self.reg.set_flag(C, v);
        self.reg.set_flag(H, false);
        self.reg.set_flag(N, false);
    }

    fn alu_scf(&mut self) {
        self.reg.set_flag(C, true);
        self.reg.set_flag(H, false);
        self.reg.set_flag(N, false);
    }

    fn alu_rlc(&mut self, a: u8) -> u8 {
        let c = (a & 0x80) >> 7 == 0x01;
        let r = (a << 1) | u8::from(c);
        self.reg.set_flag(C, c);
        self.reg.set_flag(H, false);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, r == 0x00);
        r
    }

    fn alu_rl(&mut self, a: u8) -> u8 {
        let c = (a & 0x80) >> 7 == 0x01;
        let r = (a << 1) + u8::from(self.reg.get_flag(C));
        self.reg.set_flag(C, c);
        self.reg.set_flag(H, false);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, r == 0x00);
        r
    }

    fn alu_rrc(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = if c { 0x80 | (a >> 1) } else { (a >> 1) };
        self.reg.set_flag(C, c);
        self.reg.set_flag(H, false);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, r == 0x00);
        r
    }

    fn alu_rr(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = if self.reg.get_flag(C) {
            0x80 | (a >> 1)
        } else {
            a >> 1
        };
        self.reg.set_flag(C, c);
        self.reg.set_flag(H, false);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, r == 0x00);
        r
    }

    fn alu_sla(&mut self, a: u8) -> u8 {
        let c = (a & 0x80) >> 7 == 0x01;
        let r = a << 1;
        self.reg.set_flag(C, c);
        self.reg.set_flag(H, false);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, r == 0x00);
        r
    }

    fn alu_sra(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (a & 0x80);
        self.reg.set_flag(C, c);
        self.reg.set_flag(H, false);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, r == 0x00);
        r
    }

    fn alu_srl(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = a >> 1;
        self.reg.set_flag(C, c);
        self.reg.set_flag(H, false);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, r == 0x00);
        r
    }

    fn alu_bit(&mut self, a: u8, b: u8) {
        let r = a & (1 << b) == 0x00;
        self.reg.set_flag(H, true);
        self.reg.set_flag(N, false);
        self.reg.set_flag(Z, r);
    }

    fn alu_set(&mut self, a: u8, b: u8) -> u8 {
        a | (1 << b)
    }

    fn alu_res(&mut self, a: u8, b: u8) -> u8 {
        a & !(1 << b)
    }

    fn alu_jr(&mut self, mem: &mut dyn Memory) {
        let n = mem.read(self.reg.pc) as i8;
        self.reg.pc += 1;
        self.reg.pc = (u32::from(self.reg.pc) as i32 + i32::from(n)) as u16;
    }
}
