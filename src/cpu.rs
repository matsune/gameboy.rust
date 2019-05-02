use crate::cartridge::Cartridge;
use crate::memory::Memory;
use crate::mmu::MMU;
use crate::reg::{Flag, Registers};
use crate::util::{is_bit_on, set_bit};

pub struct CPU {
    reg: Registers,
    mmu: MMU,
    enable_interrupts: bool,
}

impl CPU {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            reg: Registers::default(),
            mmu: MMU::new(cartridge),
            enable_interrupts: false,
        }
    }

    pub fn run(&mut self) {
        self.handle_interrupt();

        let opcode = self.read_byte();
        println!("OP 0x{:2x}", opcode);
        self.command(opcode);

        println!("{:?}", self.reg);
    }

    fn read_byte(&mut self) -> u8 {
        let n = self.mmu.read(self.reg.pc);
        self.reg.pc += 1;
        n
    }

    fn read_word(&mut self) -> u16 {
        let nn = self.mmu.read_word(self.reg.pc);
        self.reg.pc += 2;
        nn
    }

    fn handle_interrupt(&self) {
        // TODO:
    }

    fn push(&mut self, word: u16) {
        self.reg.sp -= 2;
        self.mmu.write_word(self.reg.sp, word);
    }

    fn pop(&mut self) -> u16 {
        let nn = self.mmu.read_word(self.reg.sp);
        self.reg.sp += 2;
        nn
    }

    fn command(&mut self, opcode: u8) {
        match opcode {
            // LD nn, n
            0x06 => self.reg.b = self.read_byte(),
            0x0e => self.reg.c = self.read_byte(),
            0x16 => self.reg.d = self.read_byte(),
            0x1e => self.reg.e = self.read_byte(),
            0x26 => self.reg.h = self.read_byte(),
            0x2e => self.reg.l = self.read_byte(),

            // LD r1, r2
            0x7f => {}
            0x78 => self.reg.a = self.reg.b,
            0x79 => self.reg.a = self.reg.c,
            0x7a => self.reg.a = self.reg.d,
            0x7b => self.reg.a = self.reg.e,
            0x7c => self.reg.a = self.reg.h,
            0x7d => self.reg.a = self.reg.l,
            0x7e => self.reg.a = self.mmu.read(self.reg.hl()),
            0x40 => self.reg.b = self.reg.b,
            0x41 => self.reg.b = self.reg.c,
            0x42 => self.reg.b = self.reg.d,
            0x43 => self.reg.b = self.reg.e,
            0x44 => self.reg.b = self.reg.h,
            0x45 => self.reg.b = self.reg.l,
            0x46 => self.reg.b = self.mmu.read(self.reg.hl()),
            0x47 => self.reg.b = self.reg.a,
            0x48 => self.reg.c = self.reg.b,
            0x49 => self.reg.c = self.reg.c,
            0x4a => self.reg.c = self.reg.d,
            0x4b => self.reg.c = self.reg.e,
            0x4c => self.reg.c = self.reg.h,
            0x4d => self.reg.c = self.reg.l,
            0x4e => self.reg.c = self.mmu.read(self.reg.hl()),
            0x4f => self.reg.c = self.reg.a,
            0x50 => self.reg.d = self.reg.b,
            0x51 => self.reg.d = self.reg.c,
            0x52 => self.reg.d = self.reg.d,
            0x53 => self.reg.d = self.reg.e,
            0x54 => self.reg.d = self.reg.h,
            0x55 => self.reg.d = self.reg.l,
            0x56 => self.reg.d = self.mmu.read(self.reg.hl()),
            0x57 => self.reg.d = self.reg.a,
            0x58 => self.reg.e = self.reg.b,
            0x59 => self.reg.e = self.reg.c,
            0x5a => self.reg.e = self.reg.d,
            0x5b => self.reg.e = self.reg.e,
            0x5c => self.reg.e = self.reg.h,
            0x5d => self.reg.e = self.reg.l,
            0x5e => self.reg.e = self.mmu.read(self.reg.hl()),
            0x5f => self.reg.e = self.reg.a,
            0x60 => self.reg.h = self.reg.b,
            0x61 => self.reg.h = self.reg.c,
            0x62 => self.reg.h = self.reg.d,
            0x63 => self.reg.h = self.reg.e,
            0x64 => self.reg.h = self.reg.h,
            0x65 => self.reg.h = self.reg.l,
            0x66 => self.reg.h = self.mmu.read(self.reg.hl()),
            0x67 => self.reg.h = self.reg.a,
            0x68 => self.reg.l = self.reg.b,
            0x69 => self.reg.l = self.reg.c,
            0x6a => self.reg.l = self.reg.d,
            0x6b => self.reg.l = self.reg.e,
            0x6c => self.reg.l = self.reg.h,
            0x6d => self.reg.l = self.reg.l,
            0x6e => self.reg.l = self.mmu.read(self.reg.hl()),
            0x6f => self.reg.l = self.reg.a,
            0x70 => self.mmu.write(self.reg.hl(), self.reg.b),
            0x71 => self.mmu.write(self.reg.hl(), self.reg.c),
            0x72 => self.mmu.write(self.reg.hl(), self.reg.d),
            0x73 => self.mmu.write(self.reg.hl(), self.reg.e),
            0x74 => self.mmu.write(self.reg.hl(), self.reg.h),
            0x75 => self.mmu.write(self.reg.hl(), self.reg.l),
            0x36 => {
                let n = self.read_byte();
                self.mmu.write(self.reg.hl(), n);
            }

            // LD A, n
            0x0a => self.reg.a = self.mmu.read(self.reg.bc()),
            0x1a => self.reg.a = self.mmu.read(self.reg.de()),
            0xfa => {
                let nn = self.read_word();
                self.reg.a = self.mmu.read(nn);
            }
            0x3e => self.reg.a = self.read_byte(),
            // LD n, A
            0x02 => self.mmu.write(self.reg.bc(), self.reg.a),
            0x12 => self.mmu.write(self.reg.de(), self.reg.a),
            0x77 => self.mmu.write(self.reg.hl(), self.reg.a),
            0xea => {
                let nn = self.read_word();
                self.mmu.write(nn, self.reg.a);
            }
            // LD A, (C)
            0xf2 => self.reg.a = self.mmu.read(0xff00 + u16::from(self.reg.c)),
            // LD (C), A
            0xe2 => self.mmu.write(0xff00 + u16::from(self.reg.c), self.reg.a),
            // LDD A, (HL)
            0x3a => {
                let hl = self.reg.decrement_hl();
                self.reg.a = self.mmu.read(hl);
            }
            // LDD (HL), A
            0x32 => {
                let hl = self.reg.decrement_hl();
                self.mmu.write(hl, self.reg.a);
            }
            // LDI A, (HL)
            0x2a => {
                let hl = self.reg.increment_hl();
                self.reg.a = self.mmu.read(hl);
            }
            // LDI (HL), A
            0x22 => {
                let hl = self.reg.increment_hl();
                self.mmu.write(hl, self.reg.a);
            }
            // LDH (n), A
            0xe0 => {
                let n = self.read_byte();
                self.mmu.write(0xff00 + u16::from(n), self.reg.a);
            }
            // LDH A, (n)
            0xf0 => {
                let n = self.read_byte();
                self.reg.a = self.mmu.read(0xff00 + u16::from(n));
            }

            // 16-Bit Loads
            // LD n, nn
            0x01 => {
                let nn = self.read_word();
                self.reg.set_bc(nn);
            }
            0x11 => {
                let nn = self.read_word();
                self.reg.set_de(nn);
            }
            0x21 => {
                let nn = self.read_word();
                self.reg.set_hl(nn);
            }
            0x31 => self.reg.sp = self.read_word(),
            // LD SP, HL
            0xf9 => self.reg.sp = self.reg.hl(),
            // LDHL SP, n
            0xf8 => {
                let a = self.reg.sp;
                let b = u16::from(self.read_byte());
                self.reg.set_flag(Flag::Z, false);
                self.reg.set_flag(Flag::N, false);
                self.reg
                    .set_flag(Flag::H, ((a & 0x000f) + (b & 0x000f)) > 0x000f);
                self.reg
                    .set_flag(Flag::C, ((a & 0x00ff) + (b & 0x00ff)) > 0x00ff);
                self.reg.set_hl(a.wrapping_add(b));
            }
            // LD (nn), SP
            0x08 => {
                let nn = self.read_word();
                self.mmu.write_word(nn, self.reg.sp);
            }
            // PUSH nn
            0xf5 => self.push(self.reg.af()),
            0xc5 => self.push(self.reg.bc()),
            0xd5 => self.push(self.reg.de()),
            0xe5 => self.push(self.reg.hl()),
            // POP nn
            0xf1 => {
                let nn = self.pop();
                self.reg.set_af(nn);
            }
            0xc1 => {
                let nn = self.pop();
                self.reg.set_bc(nn);
            }
            0xd1 => {
                let nn = self.pop();
                self.reg.set_de(nn);
            }
            0xe1 => {
                let nn = self.pop();
                self.reg.set_hl(nn);
            }

            // 8-Bit ALU
            // ADD A, n
            0x87 => self.alu_add(self.reg.a),
            0x80 => self.alu_add(self.reg.b),
            0x81 => self.alu_add(self.reg.c),
            0x82 => self.alu_add(self.reg.d),
            0x83 => self.alu_add(self.reg.e),
            0x84 => self.alu_add(self.reg.h),
            0x85 => self.alu_add(self.reg.l),
            0x86 => self.alu_add(self.mmu.read(self.reg.hl())),
            0xc6 => {
                let n = self.read_byte();
                self.alu_add(n);
            }
            // ADC A, n
            0x8f => self.alu_adc(self.reg.a),
            0x88 => self.alu_adc(self.reg.b),
            0x89 => self.alu_adc(self.reg.c),
            0x8a => self.alu_adc(self.reg.d),
            0x8b => self.alu_adc(self.reg.e),
            0x8c => self.alu_adc(self.reg.h),
            0x8d => self.alu_adc(self.reg.l),
            0x8e => self.alu_adc(self.mmu.read(self.reg.hl())),
            0xce => {
                let n = self.read_byte();
                self.alu_adc(n);
            }
            // SUB A, n
            0x97 => self.alu_sub(self.reg.a),
            0x90 => self.alu_sub(self.reg.b),
            0x91 => self.alu_sub(self.reg.c),
            0x92 => self.alu_sub(self.reg.d),
            0x93 => self.alu_sub(self.reg.e),
            0x94 => self.alu_sub(self.reg.h),
            0x95 => self.alu_sub(self.reg.l),
            0x96 => self.alu_sub(self.mmu.read(self.reg.hl())),
            0xd6 => {
                let n = self.read_byte();
                self.alu_sub(n);
            }
            // SBC A, n
            0x9f => self.alu_sbc(self.reg.a),
            0x98 => self.alu_sbc(self.reg.b),
            0x99 => self.alu_sbc(self.reg.c),
            0x9a => self.alu_sbc(self.reg.d),
            0x9b => self.alu_sbc(self.reg.e),
            0x9c => self.alu_sbc(self.reg.h),
            0x9d => self.alu_sbc(self.reg.l),
            0x9e => self.alu_sbc(self.mmu.read(self.reg.hl())),
            // AND A, n
            0xa7 => self.alu_and(self.reg.a),
            0xa0 => self.alu_and(self.reg.b),
            0xa1 => self.alu_and(self.reg.c),
            0xa2 => self.alu_and(self.reg.d),
            0xa3 => self.alu_and(self.reg.e),
            0xa4 => self.alu_and(self.reg.h),
            0xa5 => self.alu_and(self.reg.l),
            0xa6 => self.alu_and(self.mmu.read(self.reg.hl())),
            0xe6 => {
                let n = self.read_byte();
                self.alu_and(n);
            }
            // OR A, n
            0xb7 => self.alu_or(self.reg.a),
            0xb0 => self.alu_or(self.reg.b),
            0xb1 => self.alu_or(self.reg.c),
            0xb2 => self.alu_or(self.reg.d),
            0xb3 => self.alu_or(self.reg.e),
            0xb4 => self.alu_or(self.reg.h),
            0xb5 => self.alu_or(self.reg.l),
            0xb6 => self.alu_or(self.mmu.read(self.reg.hl())),
            0xf6 => {
                let n = self.read_byte();
                self.alu_or(n);
            }
            // XOR A, n
            0xaf => self.alu_xor(self.reg.a),
            0xa8 => self.alu_xor(self.reg.b),
            0xa9 => self.alu_xor(self.reg.c),
            0xaa => self.alu_xor(self.reg.d),
            0xab => self.alu_xor(self.reg.e),
            0xac => self.alu_xor(self.reg.h),
            0xad => self.alu_xor(self.reg.l),
            0xae => self.alu_xor(self.mmu.read(self.reg.hl())),
            0xee => {
                let n = self.read_byte();
                self.alu_xor(n);
            }
            // CP A, n
            0xbf => self.alu_cp(self.reg.a),
            0xb8 => self.alu_cp(self.reg.b),
            0xb9 => self.alu_cp(self.reg.c),
            0xba => self.alu_cp(self.reg.d),
            0xbb => self.alu_cp(self.reg.e),
            0xbc => self.alu_cp(self.reg.h),
            0xbd => self.alu_cp(self.reg.l),
            0xbe => self.alu_cp(self.mmu.read(self.reg.hl())),
            0xfe => {
                let n = self.read_byte();
                self.alu_cp(n);
            }
            // INC n
            0x3c => self.reg.a = self.alu_inc(self.reg.a),
            0x04 => self.reg.b = self.alu_inc(self.reg.b),
            0x0c => self.reg.c = self.alu_inc(self.reg.c),
            0x14 => self.reg.d = self.alu_inc(self.reg.d),
            0x1c => self.reg.e = self.alu_inc(self.reg.e),
            0x24 => self.reg.h = self.alu_inc(self.reg.h),
            0x2c => self.reg.l = self.alu_inc(self.reg.l),
            0x34 => {
                let hl = self.reg.hl();
                let n = self.alu_inc(self.mmu.read(hl));
                self.mmu.write(hl, n);
            }
            // DEC n
            0x3d => self.reg.a = self.alu_dec(self.reg.a),
            0x05 => self.reg.b = self.alu_dec(self.reg.b),
            0x0d => self.reg.c = self.alu_dec(self.reg.c),
            0x15 => self.reg.d = self.alu_dec(self.reg.d),
            0x1d => self.reg.e = self.alu_dec(self.reg.e),
            0x25 => self.reg.h = self.alu_dec(self.reg.h),
            0x2d => self.reg.l = self.alu_dec(self.reg.l),
            0x35 => {
                let hl = self.reg.hl();
                let n = self.alu_dec(self.mmu.read(hl));
                self.mmu.write(hl, n);
            }
            // 16-Bit Arithmetic
            // ADD HL, n
            0x09 => self.alu_add_hl(self.reg.bc()),
            0x19 => self.alu_add_hl(self.reg.de()),
            0x29 => self.alu_add_hl(self.reg.hl()),
            0x39 => self.alu_add_hl(self.reg.sp),
            // ADD SP, n
            0xe8 => {
                let n = self.read_byte();
                self.alu_add_sp(n);
            }
            // INC nn
            0x03 => self.reg.set_bc(self.reg.bc().wrapping_add(1)),
            0x13 => self.reg.set_de(self.reg.de().wrapping_add(1)),
            0x23 => self.reg.set_hl(self.reg.hl().wrapping_add(1)),
            0x33 => self.reg.sp = self.reg.sp.wrapping_add(1),
            // DEC nn
            0x0b => self.reg.set_bc(self.reg.bc().wrapping_sub(1)),
            0x1b => self.reg.set_de(self.reg.de().wrapping_sub(1)),
            0x2b => self.reg.set_hl(self.reg.hl().wrapping_sub(1)),
            0x3b => self.reg.sp = self.reg.sp.wrapping_sub(1),

            // DAA
            0x27 => {
                let mut a = self.reg.a;
                if (a & 0x0f) > 9 || self.reg.get_flag(Flag::H) {
                    a = a.wrapping_add(0x06);
                }
                if (a & 0xf0) > 0x90 || self.reg.get_flag(Flag::C) {
                    a = a.wrapping_add(0x60);
                    self.reg.set_flag(Flag::C, true);
                }
                self.reg.set_flag(Flag::Z, a == 0);
                self.reg.set_flag(Flag::H, false);
                self.reg.a = a;
            }
            // CPL
            0x2f => {
                self.reg.set_flag(Flag::N, true);
                self.reg.set_flag(Flag::H, true);
                self.reg.a = !self.reg.a;
            }
            // CCF
            0x3f => {
                self.reg.set_flag(Flag::N, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, !self.reg.get_flag(Flag::C));
            }
            // SCF
            0x37 => {
                self.reg.set_flag(Flag::N, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, true);
            }
            // NOP
            0x00 => {}
            // HALT
            0x76 => unimplemented!("HALT"),
            // STOP
            0x10 => unimplemented!("STOP"),
            // DI
            0xf3 => unimplemented!("DI"),
            // EI
            0xfb => unimplemented!("EI"),

            // RLCA
            0x07 => self.reg.a = self.alu_rlc(self.reg.a),
            // RLA
            0x17 => self.reg.a = self.alu_rl(self.reg.a),
            // RRCA
            0x0f => self.reg.a = self.alu_rrc(self.reg.a),
            // RRA
            0x1f => self.reg.a = self.alu_rr(self.reg.a),

            // JP nn
            0xc3 => self.jump(),
            // JP NZ, nn
            0xc2 => {
                // Jump if Z flag is reset
                if !self.reg.get_flag(Flag::Z) {
                    self.jump();
                }
            }
            // JP Z, nn
            0xca => {
                // Jump if Z flag is set
                if self.reg.get_flag(Flag::Z) {
                    self.jump();
                }
            }
            // JP NC, nn
            0xd2 => {
                // Jump if C flag is reset
                if !self.reg.get_flag(Flag::C) {
                    self.jump();
                }
            }
            // JP C, nn
            0xda => {
                // Jump if C flag is set
                if self.reg.get_flag(Flag::C) {
                    self.jump();
                }
            }
            // JP (HL)
            0xe9 => self.reg.pc = self.reg.hl(),
            // JR n
            0x18 => self.alu_jr(),
            // JR NZ, n
            0x20 => {
                if !self.reg.get_flag(Flag::Z) {
                    self.alu_jr();
                } else {
                    self.reg.pc += 1;
                }
            }
            // JR Z, n
            0x28 => {
                if self.reg.get_flag(Flag::Z) {
                    self.alu_jr();
                } else {
                    self.reg.pc += 1;
                }
            }
            // JR NC, n
            0x30 => {
                if !self.reg.get_flag(Flag::C) {
                    self.alu_jr();
                } else {
                    self.reg.pc += 1;
                }
            }
            // JR C, n
            0x38 => {
                if self.reg.get_flag(Flag::C) {
                    self.alu_jr();
                } else {
                    self.reg.pc += 1;
                }
            }
            // CALL nn
            0xcd => {
                self.push(self.reg.pc + 2);
                self.reg.pc = self.mmu.read_word(self.reg.pc);
            }
            // CALL NZ, nn
            0xc4 => {
                if !self.reg.get_flag(Flag::Z) {
                    self.push(self.reg.pc + 2);
                    self.reg.pc = self.mmu.read_word(self.reg.pc);
                } else {
                    self.reg.pc += 2;
                }
            }
            // CALL Z, nn
            0xcc => {
                if self.reg.get_flag(Flag::Z) {
                    self.push(self.reg.pc + 2);
                    self.reg.pc = self.mmu.read_word(self.reg.pc);
                } else {
                    self.reg.pc += 2;
                }
            }
            // CALL NC, nn
            0xd4 => {
                if !self.reg.get_flag(Flag::C) {
                    self.push(self.reg.pc + 2);
                    self.reg.pc = self.mmu.read_word(self.reg.pc);
                } else {
                    self.reg.pc += 2;
                }
            }
            // CALL C, nn
            0xdc => {
                if self.reg.get_flag(Flag::C) {
                    self.push(self.reg.pc + 2);
                    self.reg.pc = self.mmu.read_word(self.reg.pc);
                } else {
                    self.reg.pc += 2;
                }
            }
            // RST n
            0xc7 => {
                self.push(self.reg.pc);
                self.reg.pc = 0x00;
            }
            0xcf => {
                self.push(self.reg.pc);
                self.reg.pc = 0x08;
            }
            0xd7 => {
                self.push(self.reg.pc);
                self.reg.pc = 0x10;
            }
            0xdf => {
                self.push(self.reg.pc);
                self.reg.pc = 0x18;
            }
            0xe7 => {
                self.push(self.reg.pc);
                self.reg.pc = 0x20;
            }
            0xef => {
                self.push(self.reg.pc);
                self.reg.pc = 0x28;
            }
            0xf7 => {
                self.push(self.reg.pc);
                self.reg.pc = 0x30;
            }
            0xff => {
                self.push(self.reg.pc);
                self.reg.pc = 0x38;
            }
            // RET
            0xc9 => self.reg.pc = self.pop(),
            // RET NZ
            0xc0 => {
                if !self.reg.get_flag(Flag::Z) {
                    self.reg.pc = self.pop();
                }
            }
            // RET Z
            0xc8 => {
                if self.reg.get_flag(Flag::Z) {
                    self.reg.pc = self.pop();
                }
            }
            // RET NC
            0xd0 => {
                if !self.reg.get_flag(Flag::C) {
                    self.reg.pc = self.pop();
                }
            }
            // RET C
            0xd8 => {
                if self.reg.get_flag(Flag::C) {
                    self.reg.pc = self.pop();
                }
            }
            // RETI
            0xd9 => {
                self.reg.pc = self.pop();
                self.enable_interrupts = true;
            }

            0xcb => {
                let opcode = self.read_byte();
                println!("OP 0xCB{:2x}", opcode);
                self.ext_command(opcode);
            }
            0xde => unimplemented!("0xde"),
            0xd3 | 0xdb | 0xdd | 0xe3 | 0xe4 | 0xeb | 0xec | 0xed | 0xf4 | 0xfc | 0xfd => {
                panic!("unknown instruction 0x{:x}", opcode)
            }
        }
    }

    fn ext_command(&mut self, opcode: u8) {
        match opcode {
            // RLC
            0x07 => self.reg.a = self.alu_rlc(self.reg.a),
            0x00 => self.reg.b = self.alu_rlc(self.reg.b),
            0x01 => self.reg.c = self.alu_rlc(self.reg.c),
            0x02 => self.reg.d = self.alu_rlc(self.reg.d),
            0x03 => self.reg.e = self.alu_rlc(self.reg.e),
            0x04 => self.reg.h = self.alu_rlc(self.reg.h),
            0x05 => self.reg.l = self.alu_rlc(self.reg.l),
            0x06 => {
                let hl = self.reg.hl();
                let n = self.alu_rlc(self.mmu.read(hl));
                self.mmu.write(hl, n);
            }
            // RL
            0x17 => self.reg.a = self.alu_rl(self.reg.a),
            0x10 => self.reg.b = self.alu_rl(self.reg.b),
            0x11 => self.reg.c = self.alu_rl(self.reg.c),
            0x12 => self.reg.d = self.alu_rl(self.reg.d),
            0x13 => self.reg.e = self.alu_rl(self.reg.e),
            0x14 => self.reg.h = self.alu_rl(self.reg.h),
            0x15 => self.reg.l = self.alu_rl(self.reg.l),
            0x16 => {
                let hl = self.reg.hl();
                let n = self.alu_rl(self.mmu.read(hl));
                self.mmu.write(hl, n);
            }
            // RRC
            0x0f => self.reg.a = self.alu_rrc(self.reg.a),
            0x08 => self.reg.b = self.alu_rrc(self.reg.b),
            0x09 => self.reg.c = self.alu_rrc(self.reg.c),
            0x0a => self.reg.d = self.alu_rrc(self.reg.d),
            0x0b => self.reg.e = self.alu_rrc(self.reg.e),
            0x0c => self.reg.h = self.alu_rrc(self.reg.h),
            0x0d => self.reg.l = self.alu_rrc(self.reg.l),
            0x0e => {
                let hl = self.reg.hl();
                let n = self.alu_rrc(self.mmu.read(hl));
                self.mmu.write(hl, n);
            }
            // RR
            0x1f => self.reg.a = self.alu_rr(self.reg.a),
            0x18 => self.reg.b = self.alu_rr(self.reg.b),
            0x19 => self.reg.c = self.alu_rr(self.reg.c),
            0x1a => self.reg.d = self.alu_rr(self.reg.d),
            0x1b => self.reg.e = self.alu_rr(self.reg.e),
            0x1c => self.reg.h = self.alu_rr(self.reg.h),
            0x1d => self.reg.l = self.alu_rr(self.reg.l),
            0x1e => {
                let hl = self.reg.hl();
                let n = self.alu_rr(self.mmu.read(hl));
                self.mmu.write(hl, n);
            }
            // SLA
            0x27 => self.reg.a = self.alu_sla(self.reg.a),
            0x20 => self.reg.b = self.alu_sla(self.reg.b),
            0x21 => self.reg.c = self.alu_sla(self.reg.c),
            0x22 => self.reg.d = self.alu_sla(self.reg.d),
            0x23 => self.reg.e = self.alu_sla(self.reg.e),
            0x24 => self.reg.h = self.alu_sla(self.reg.h),
            0x25 => self.reg.l = self.alu_sla(self.reg.l),
            0x26 => {
                let hl = self.reg.hl();
                let n = self.alu_sla(self.mmu.read(hl));
                self.mmu.write(hl, n);
            }
            // SRA
            0x2f => self.reg.a = self.alu_sra(self.reg.a),
            0x28 => self.reg.b = self.alu_sra(self.reg.b),
            0x29 => self.reg.c = self.alu_sra(self.reg.c),
            0x2a => self.reg.d = self.alu_sra(self.reg.d),
            0x2b => self.reg.e = self.alu_sra(self.reg.e),
            0x2c => self.reg.h = self.alu_sra(self.reg.h),
            0x2d => self.reg.l = self.alu_sra(self.reg.l),
            0x2e => {
                let hl = self.reg.hl();
                let n = self.alu_sra(self.mmu.read(hl));
                self.mmu.write(hl, n);
            }
            // SRL
            0x3f => self.reg.a = self.alu_srl(self.reg.a),
            0x38 => self.reg.b = self.alu_srl(self.reg.b),
            0x39 => self.reg.c = self.alu_srl(self.reg.c),
            0x3a => self.reg.d = self.alu_srl(self.reg.d),
            0x3b => self.reg.e = self.alu_srl(self.reg.e),
            0x3c => self.reg.h = self.alu_srl(self.reg.h),
            0x3d => self.reg.l = self.alu_srl(self.reg.l),
            0x3e => {
                let hl = self.reg.hl();
                let n = self.alu_srl(self.mmu.read(hl));
                self.mmu.write(hl, n);
            }
            // Swap
            0x37 => self.reg.a = self.swap(self.reg.a),
            0x30 => self.reg.b = self.swap(self.reg.b),
            0x31 => self.reg.c = self.swap(self.reg.c),
            0x32 => self.reg.d = self.swap(self.reg.d),
            0x33 => self.reg.e = self.swap(self.reg.e),
            0x34 => self.reg.h = self.swap(self.reg.h),
            0x35 => self.reg.l = self.swap(self.reg.l),
            0x36 => {
                let hl = self.reg.hl();
                let n = self.swap(self.mmu.read(hl));
                self.mmu.write(hl, n);
            }
            // Bit
            0x40 => self.alu_bit(0, self.reg.b),
            0x41 => self.alu_bit(0, self.reg.c),
            0x42 => self.alu_bit(0, self.reg.d),
            0x43 => self.alu_bit(0, self.reg.e),
            0x44 => self.alu_bit(0, self.reg.h),
            0x45 => self.alu_bit(0, self.reg.l),
            0x46 => self.alu_bit(0, self.mmu.read(self.reg.hl())),
            0x47 => self.alu_bit(0, self.reg.a),
            0x48 => self.alu_bit(1, self.reg.b),
            0x49 => self.alu_bit(1, self.reg.c),
            0x4a => self.alu_bit(1, self.reg.d),
            0x4b => self.alu_bit(1, self.reg.e),
            0x4c => self.alu_bit(1, self.reg.h),
            0x4d => self.alu_bit(1, self.reg.l),
            0x4e => self.alu_bit(1, self.mmu.read(self.reg.hl())),
            0x4f => self.alu_bit(1, self.reg.a),
            0x50 => self.alu_bit(2, self.reg.b),
            0x51 => self.alu_bit(2, self.reg.c),
            0x52 => self.alu_bit(2, self.reg.d),
            0x53 => self.alu_bit(2, self.reg.e),
            0x54 => self.alu_bit(2, self.reg.h),
            0x55 => self.alu_bit(2, self.reg.l),
            0x56 => self.alu_bit(2, self.mmu.read(self.reg.hl())),
            0x57 => self.alu_bit(2, self.reg.a),
            0x58 => self.alu_bit(3, self.reg.b),
            0x59 => self.alu_bit(3, self.reg.c),
            0x5a => self.alu_bit(3, self.reg.d),
            0x5b => self.alu_bit(3, self.reg.e),
            0x5c => self.alu_bit(3, self.reg.h),
            0x5d => self.alu_bit(3, self.reg.l),
            0x5e => self.alu_bit(3, self.mmu.read(self.reg.hl())),
            0x5f => self.alu_bit(3, self.reg.a),
            0x60 => self.alu_bit(4, self.reg.b),
            0x61 => self.alu_bit(4, self.reg.c),
            0x62 => self.alu_bit(4, self.reg.d),
            0x63 => self.alu_bit(4, self.reg.e),
            0x64 => self.alu_bit(4, self.reg.h),
            0x65 => self.alu_bit(4, self.reg.l),
            0x66 => self.alu_bit(4, self.mmu.read(self.reg.hl())),
            0x67 => self.alu_bit(4, self.reg.a),
            0x68 => self.alu_bit(5, self.reg.b),
            0x69 => self.alu_bit(5, self.reg.c),
            0x6a => self.alu_bit(5, self.reg.d),
            0x6b => self.alu_bit(5, self.reg.e),
            0x6c => self.alu_bit(5, self.reg.h),
            0x6d => self.alu_bit(5, self.reg.l),
            0x6e => self.alu_bit(5, self.mmu.read(self.reg.hl())),
            0x6f => self.alu_bit(5, self.reg.a),
            0x70 => self.alu_bit(6, self.reg.b),
            0x71 => self.alu_bit(6, self.reg.c),
            0x72 => self.alu_bit(6, self.reg.d),
            0x73 => self.alu_bit(6, self.reg.e),
            0x74 => self.alu_bit(6, self.reg.h),
            0x75 => self.alu_bit(6, self.reg.l),
            0x76 => self.alu_bit(6, self.mmu.read(self.reg.hl())),
            0x77 => self.alu_bit(6, self.reg.a),
            0x78 => self.alu_bit(7, self.reg.b),
            0x79 => self.alu_bit(7, self.reg.c),
            0x7a => self.alu_bit(7, self.reg.d),
            0x7b => self.alu_bit(7, self.reg.e),
            0x7c => self.alu_bit(7, self.reg.h),
            0x7d => self.alu_bit(7, self.reg.l),
            0x7e => self.alu_bit(7, self.mmu.read(self.reg.hl())),
            0x7f => self.alu_bit(7, self.reg.a),
            // SET
            0xc7 => self.reg.a = set_bit(self.reg.a, 0, true),
            0xc0 => self.reg.b = set_bit(self.reg.b, 0, true),
            0xc1 => self.reg.c = set_bit(self.reg.c, 0, true),
            0xc2 => self.reg.d = set_bit(self.reg.d, 0, true),
            0xc3 => self.reg.e = set_bit(self.reg.e, 0, true),
            0xc4 => self.reg.h = set_bit(self.reg.h, 0, true),
            0xc5 => self.reg.l = set_bit(self.reg.l, 0, true),
            0xc6 => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 0, true);
                self.mmu.write(hl, b);
            }
            0xcf => self.reg.a = set_bit(self.reg.a, 1, true),
            0xc8 => self.reg.b = set_bit(self.reg.b, 1, true),
            0xc9 => self.reg.c = set_bit(self.reg.c, 1, true),
            0xca => self.reg.d = set_bit(self.reg.d, 1, true),
            0xcb => self.reg.e = set_bit(self.reg.e, 1, true),
            0xcc => self.reg.h = set_bit(self.reg.h, 1, true),
            0xcd => self.reg.l = set_bit(self.reg.l, 1, true),
            0xce => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 1, true);
                self.mmu.write(hl, b);
            }
            0xd7 => self.reg.a = set_bit(self.reg.a, 2, true),
            0xd0 => self.reg.b = set_bit(self.reg.b, 2, true),
            0xd1 => self.reg.c = set_bit(self.reg.c, 2, true),
            0xd2 => self.reg.d = set_bit(self.reg.d, 2, true),
            0xd3 => self.reg.e = set_bit(self.reg.e, 2, true),
            0xd4 => self.reg.h = set_bit(self.reg.h, 2, true),
            0xd5 => self.reg.l = set_bit(self.reg.l, 2, true),
            0xd6 => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 2, true);
                self.mmu.write(hl, b);
            }
            0xdf => self.reg.a = set_bit(self.reg.a, 3, true),
            0xd8 => self.reg.b = set_bit(self.reg.b, 3, true),
            0xd9 => self.reg.c = set_bit(self.reg.c, 3, true),
            0xda => self.reg.d = set_bit(self.reg.d, 3, true),
            0xdb => self.reg.e = set_bit(self.reg.e, 3, true),
            0xdc => self.reg.h = set_bit(self.reg.h, 3, true),
            0xdd => self.reg.l = set_bit(self.reg.l, 3, true),
            0xde => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 3, true);
                self.mmu.write(hl, b);
            }
            0xe7 => self.reg.a = set_bit(self.reg.a, 4, true),
            0xe0 => self.reg.b = set_bit(self.reg.b, 4, true),
            0xe1 => self.reg.c = set_bit(self.reg.c, 4, true),
            0xe2 => self.reg.d = set_bit(self.reg.d, 4, true),
            0xe3 => self.reg.e = set_bit(self.reg.e, 4, true),
            0xe4 => self.reg.h = set_bit(self.reg.h, 4, true),
            0xe5 => self.reg.l = set_bit(self.reg.l, 4, true),
            0xe6 => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 4, true);
                self.mmu.write(hl, b);
            }
            0xef => self.reg.a = set_bit(self.reg.a, 5, true),
            0xe8 => self.reg.b = set_bit(self.reg.b, 5, true),
            0xe9 => self.reg.c = set_bit(self.reg.c, 5, true),
            0xea => self.reg.d = set_bit(self.reg.d, 5, true),
            0xeb => self.reg.e = set_bit(self.reg.e, 5, true),
            0xec => self.reg.h = set_bit(self.reg.h, 5, true),
            0xed => self.reg.l = set_bit(self.reg.l, 5, true),
            0xee => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 5, true);
                self.mmu.write(hl, b);
            }
            0xf7 => self.reg.a = set_bit(self.reg.a, 6, true),
            0xf0 => self.reg.b = set_bit(self.reg.b, 6, true),
            0xf1 => self.reg.c = set_bit(self.reg.c, 6, true),
            0xf2 => self.reg.d = set_bit(self.reg.d, 6, true),
            0xf3 => self.reg.e = set_bit(self.reg.e, 6, true),
            0xf4 => self.reg.h = set_bit(self.reg.h, 6, true),
            0xf5 => self.reg.l = set_bit(self.reg.l, 6, true),
            0xf6 => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 6, true);
                self.mmu.write(hl, b);
            }
            0xff => self.reg.a = set_bit(self.reg.a, 7, true),
            0xf8 => self.reg.b = set_bit(self.reg.b, 7, true),
            0xf9 => self.reg.c = set_bit(self.reg.c, 7, true),
            0xfa => self.reg.d = set_bit(self.reg.d, 7, true),
            0xfb => self.reg.e = set_bit(self.reg.e, 7, true),
            0xfc => self.reg.h = set_bit(self.reg.h, 7, true),
            0xfd => self.reg.l = set_bit(self.reg.l, 7, true),
            0xfe => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 7, true);
                self.mmu.write(hl, b);
            }
            // RES
            0x87 => self.reg.a = set_bit(self.reg.a, 0, false),
            0x80 => self.reg.b = set_bit(self.reg.b, 0, false),
            0x81 => self.reg.c = set_bit(self.reg.c, 0, false),
            0x82 => self.reg.d = set_bit(self.reg.d, 0, false),
            0x83 => self.reg.e = set_bit(self.reg.e, 0, false),
            0x84 => self.reg.h = set_bit(self.reg.h, 0, false),
            0x85 => self.reg.l = set_bit(self.reg.l, 0, false),
            0x86 => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 0, false);
                self.mmu.write(hl, b);
            }
            0x8f => self.reg.a = set_bit(self.reg.a, 1, false),
            0x88 => self.reg.b = set_bit(self.reg.b, 1, false),
            0x89 => self.reg.c = set_bit(self.reg.c, 1, false),
            0x8a => self.reg.d = set_bit(self.reg.d, 1, false),
            0x8b => self.reg.e = set_bit(self.reg.e, 1, false),
            0x8c => self.reg.h = set_bit(self.reg.h, 1, false),
            0x8d => self.reg.l = set_bit(self.reg.l, 1, false),
            0x8e => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 1, false);
                self.mmu.write(hl, b);
            }
            0x97 => self.reg.a = set_bit(self.reg.a, 2, false),
            0x90 => self.reg.b = set_bit(self.reg.b, 2, false),
            0x91 => self.reg.c = set_bit(self.reg.c, 2, false),
            0x92 => self.reg.d = set_bit(self.reg.d, 2, false),
            0x93 => self.reg.e = set_bit(self.reg.e, 2, false),
            0x94 => self.reg.h = set_bit(self.reg.h, 2, false),
            0x95 => self.reg.l = set_bit(self.reg.l, 2, false),
            0x96 => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 2, false);
                self.mmu.write(hl, b);
            }
            0x9f => self.reg.a = set_bit(self.reg.a, 3, false),
            0x98 => self.reg.b = set_bit(self.reg.b, 3, false),
            0x99 => self.reg.c = set_bit(self.reg.c, 3, false),
            0x9a => self.reg.d = set_bit(self.reg.d, 3, false),
            0x9b => self.reg.e = set_bit(self.reg.e, 3, false),
            0x9c => self.reg.h = set_bit(self.reg.h, 3, false),
            0x9d => self.reg.l = set_bit(self.reg.l, 3, false),
            0x9e => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 3, false);
                self.mmu.write(hl, b);
            }
            0xa7 => self.reg.a = set_bit(self.reg.a, 4, false),
            0xa0 => self.reg.b = set_bit(self.reg.b, 4, false),
            0xa1 => self.reg.c = set_bit(self.reg.c, 4, false),
            0xa2 => self.reg.d = set_bit(self.reg.d, 4, false),
            0xa3 => self.reg.e = set_bit(self.reg.e, 4, false),
            0xa4 => self.reg.h = set_bit(self.reg.h, 4, false),
            0xa5 => self.reg.l = set_bit(self.reg.l, 4, false),
            0xa6 => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 4, false);
                self.mmu.write(hl, b);
            }
            0xaf => self.reg.a = set_bit(self.reg.a, 5, false),
            0xa8 => self.reg.b = set_bit(self.reg.b, 5, false),
            0xa9 => self.reg.c = set_bit(self.reg.c, 5, false),
            0xaa => self.reg.d = set_bit(self.reg.d, 5, false),
            0xab => self.reg.e = set_bit(self.reg.e, 5, false),
            0xac => self.reg.h = set_bit(self.reg.h, 5, false),
            0xad => self.reg.l = set_bit(self.reg.l, 5, false),
            0xae => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 5, false);
                self.mmu.write(hl, b);
            }
            0xb7 => self.reg.a = set_bit(self.reg.a, 6, false),
            0xb0 => self.reg.b = set_bit(self.reg.b, 6, false),
            0xb1 => self.reg.c = set_bit(self.reg.c, 6, false),
            0xb2 => self.reg.d = set_bit(self.reg.d, 6, false),
            0xb3 => self.reg.e = set_bit(self.reg.e, 6, false),
            0xb4 => self.reg.h = set_bit(self.reg.h, 6, false),
            0xb5 => self.reg.l = set_bit(self.reg.l, 6, false),
            0xb6 => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 6, false);
                self.mmu.write(hl, b);
            }
            0xbf => self.reg.a = set_bit(self.reg.a, 7, false),
            0xb8 => self.reg.b = set_bit(self.reg.b, 7, false),
            0xb9 => self.reg.c = set_bit(self.reg.c, 7, false),
            0xba => self.reg.d = set_bit(self.reg.d, 7, false),
            0xbb => self.reg.e = set_bit(self.reg.e, 7, false),
            0xbc => self.reg.h = set_bit(self.reg.h, 7, false),
            0xbd => self.reg.l = set_bit(self.reg.l, 7, false),
            0xbe => {
                let hl = self.reg.hl();
                let b = set_bit(self.mmu.read(hl), 7, false);
                self.mmu.write(hl, b);
            }
        }
    }

    fn swap(&mut self, n: u8) -> u8 {
        let upper = n & 0xf0;
        let lower = n & 0x0f;
        let r = (lower << 4) | (upper >> 4);
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::C, false);
        r
    }

    fn alu_add(&mut self, n: u8) {
        let a = self.reg.a;
        let r = a.wrapping_add(n);
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, ((a & 0x0f) + (n & 0x0f)) > 0x0f);
        self.reg
            .set_flag(Flag::C, (u16::from(a) + u16::from(n)) > 0xff);
        self.reg.a = r;
    }

    fn alu_adc(&mut self, n: u8) {
        let a = self.reg.a;
        let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 };
        let r = a.wrapping_add(n).wrapping_add(c);
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg
            .set_flag(Flag::H, ((a & 0x0f) + (n & 0x0f) + (c & 0x0f)) > 0x0f);
        self.reg
            .set_flag(Flag::C, (u16::from(a) + u16::from(n) + u16::from(c)) > 0xff);
        self.reg.a = r;
    }

    fn alu_sub(&mut self, n: u8) {
        let a = self.reg.a;
        let r = a.wrapping_sub(n);
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::N, true);
        self.reg.set_flag(Flag::H, (a & 0x0f) < (n & 0x0f));
        self.reg.set_flag(Flag::C, u16::from(a) < u16::from(n));
        self.reg.a = r;
    }

    fn alu_sbc(&mut self, n: u8) {
        let a = self.reg.a;
        let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 };
        let r = a.wrapping_sub(n).wrapping_sub(c);
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::N, true);
        self.reg.set_flag(Flag::H, (a & 0x0f) < (n & 0x0f) + c);
        self.reg
            .set_flag(Flag::C, u16::from(a) < u16::from(n) + u16::from(c));
        self.reg.a = r;
    }

    fn alu_and(&mut self, n: u8) {
        let r = self.reg.a & n;
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, true);
        self.reg.set_flag(Flag::C, false);
        self.reg.a = r;
    }

    fn alu_or(&mut self, n: u8) {
        let r = self.reg.a | n;
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::C, false);
        self.reg.a = r;
    }

    fn alu_xor(&mut self, n: u8) {
        let r = self.reg.a ^ n;
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::C, false);
        self.reg.a = r;
    }

    fn alu_cp(&mut self, n: u8) {
        let r = self.reg.a;
        self.alu_sub(n);
        self.reg.a = r;
    }

    fn alu_inc(&mut self, n: u8) -> u8 {
        let r = n.wrapping_add(1);
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, (n & 0x0f) + 0x01 > 0x0f);
        r
    }

    fn alu_dec(&mut self, n: u8) -> u8 {
        let r = n.wrapping_sub(1);
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::N, true);
        self.reg.set_flag(Flag::H, (n & 0x0f) == 0);
        r
    }

    fn alu_add_hl(&mut self, nn: u16) {
        let hl = self.reg.hl();
        let r = hl.wrapping_add(nn);
        self.reg.set_flag(Flag::N, false);
        self.reg
            .set_flag(Flag::H, ((hl & 0x07ff) + (nn & 0x07ff)) > 0x07ff);
        self.reg.set_flag(Flag::C, hl > 0xffff - nn);
        self.reg.set_hl(r);
    }

    fn alu_add_sp(&mut self, n: u8) {
        let n = u16::from(n);
        let sp = self.reg.sp;
        let r = sp.wrapping_add(n);
        self.reg.set_flag(Flag::Z, false);
        self.reg.set_flag(Flag::N, false);
        self.reg
            .set_flag(Flag::H, (sp & 0x000f) + (n & 0x000f) > 0x000f);
        self.reg
            .set_flag(Flag::C, (sp & 0x00ff) + (n & 0x00ff) > 0x00ff);
        self.reg.set_hl(r);
    }

    fn alu_bit(&mut self, bit: u8, n: u8) {
        self.reg.set_flag(Flag::Z, !is_bit_on(n, bit));
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, true);
    }

    fn alu_rlc(&mut self, n: u8) -> u8 {
        let mut r = n << 1;
        if is_bit_on(n, 7) {
            r |= 1;
            self.reg.set_flag(Flag::C, true);
        } else {
            self.reg.set_flag(Flag::C, false);
        }
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::N, false);
        r
    }

    fn alu_rl(&mut self, n: u8) -> u8 {
        let mut r = n << 1;
        r |= if self.reg.get_flag(Flag::C) { 1 } else { 0 };
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::C, is_bit_on(n, 7));
        r
    }

    fn alu_rrc(&mut self, n: u8) -> u8 {
        let mut r = n >> 1;
        if is_bit_on(n, 0) {
            r |= 1 << 7;
            self.reg.set_flag(Flag::C, true);
        } else {
            self.reg.set_flag(Flag::C, false);
        }
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::N, false);
        r
    }

    fn alu_rr(&mut self, n: u8) -> u8 {
        let mut r = n >> 1;
        r |= if self.reg.get_flag(Flag::C) {
            1 << 7
        } else {
            0
        };
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::C, is_bit_on(n, 0));
        r
    }

    fn alu_sla(&mut self, n: u8) -> u8 {
        let r = n << 1;
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::C, is_bit_on(n, 7));
        r
    }

    fn alu_sra(&mut self, n: u8) -> u8 {
        let r = n >> 1 | (n & (1 << 7));
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::C, is_bit_on(n, 0));
        r
    }

    fn alu_srl(&mut self, n: u8) -> u8 {
        let r = n >> 1;
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::C, is_bit_on(n, 0));
        r
    }

    fn jump(&mut self) {
        self.reg.pc = self.read_word();
    }

    fn alu_jr(&mut self) {
        let n = self.mmu.read(self.reg.pc) as i8;
        self.reg.pc += 1;
        self.reg.pc = ((self.reg.pc as u32 as i32) + (n as i32)) as u16;
    }
}
