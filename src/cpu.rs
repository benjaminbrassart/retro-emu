use crate::bus::{Bus, BusExt};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CpuFlags {
    pub zero: bool,
    pub sub: bool,
    pub half: bool,
    pub carry: bool,
}

impl CpuFlags {
    const Z: u8 = 0b1000_0000;
    const N: u8 = 0b0100_0000;
    const H: u8 = 0b0010_0000;
    const C: u8 = 0b0001_0000;
}

impl From<u8> for CpuFlags {
    fn from(b: u8) -> Self {
        Self {
            zero: b & Self::Z == Self::Z,
            sub: b & Self::N == Self::N,
            half: b & Self::H == Self::H,
            carry: b & Self::C == Self::C,
        }
    }
}

impl Into<u8> for CpuFlags {
    fn into(self) -> u8 {
        let zero = if self.zero { Self::Z } else { 0 };
        let sub = if self.sub { Self::N } else { 0 };
        let half = if self.half { Self::H } else { 0 };
        let carry = if self.carry { Self::C } else { 0 };

        zero | sub | half | carry
    }
}

#[derive(Default)]
pub struct Cpu {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
}

impl Cpu {
    pub fn get_af(&self) -> u16 {
        u16::from_be_bytes([self.a, self.f])
    }

    pub fn set_af(&mut self, value: u16) {
        let [a, f] = value.to_be_bytes();

        self.a = a;
        self.f = f & 0xf0;
    }

    pub fn get_bc(&self) -> u16 {
        u16::from_be_bytes([self.b, self.c])
    }

    pub fn set_bc(&mut self, value: u16) {
        let [b, c] = value.to_be_bytes();

        self.b = b;
        self.c = c;
    }

    pub fn get_de(&self) -> u16 {
        u16::from_be_bytes([self.d, self.e])
    }

    pub fn set_de(&mut self, value: u16) {
        let [d, e] = value.to_be_bytes();

        self.d = d;
        self.e = e;
    }

    pub fn get_hl(&self) -> u16 {
        u16::from_be_bytes([self.h, self.l])
    }

    pub fn set_hl(&mut self, value: u16) {
        let [h, l] = value.to_be_bytes();

        self.h = h;
        self.l = l;
    }

    pub fn get_flags(&self) -> CpuFlags {
        self.f.into()
    }

    pub fn set_flags(&mut self, flags: CpuFlags) {
        self.f = flags.into()
    }

    pub fn update_flags<F>(&mut self, f: F)
    where
        F: FnOnce(&mut CpuFlags),
    {
        let mut flags = self.get_flags();

        f(&mut flags);

        self.set_flags(flags);
    }

    pub fn check_jump_condition(&self, condition: Option<JumpCondition>) -> bool {
        let flags = self.get_flags();

        match condition {
            None => true,
            Some(JumpCondition::NZ) => !flags.zero,
            Some(JumpCondition::Z) => flags.zero,
            Some(JumpCondition::NC) => !flags.carry,
            Some(JumpCondition::C) => flags.carry,
        }
    }

    pub fn jump_relative(&mut self, offset: i8) {
        self.pc = self.pc.wrapping_add_signed(offset.into());
    }

    pub fn jump_absolute(&mut self, address: u16) {
        self.pc = address;
    }

    pub fn push<B>(&mut self, bus: &mut B, value: u16)
    where
        B: Bus,
    {
        let buf = value.to_le_bytes();
        let sp = self.sp.wrapping_sub(1);

        bus.write_byte(sp, buf[0]);

        let sp = sp.wrapping_sub(1);

        bus.write_byte(sp, buf[1]);

        self.sp = sp;
    }

    pub fn pop<B>(&mut self, bus: &B) -> u16
    where
        B: Bus,
    {
        let sp = self.sp;
        let lo = bus.read_byte(sp);

        let sp = sp.wrapping_sub(1);
        let hi = bus.read_byte(sp);

        self.sp = sp.wrapping_sub(1);

        u16::from_le_bytes([lo, hi])
    }

    pub fn call<B>(&mut self, bus: &mut B, address: u16)
    where
        B: Bus,
    {
        self.push(bus, address);
        self.jump_absolute(address);
    }

    pub fn ret<B>(&mut self, bus: &B)
    where
        B: Bus,
    {
        let address = self.pop(bus);

        self.jump_absolute(address);
    }

    pub fn get_reg8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::H => self.h,
            Reg8::L => self.l,
        }
    }

    pub fn set_reg8(&mut self, reg: Reg8, value: u8) {
        match reg {
            Reg8::A => self.a = value,
            Reg8::B => self.b = value,
            Reg8::C => self.c = value,
            Reg8::D => self.d = value,
            Reg8::E => self.e = value,
            Reg8::H => self.h = value,
            Reg8::L => self.l = value,
        }
    }

    pub fn get_reg16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::AF => self.get_af(),
            Reg16::BC => self.get_bc(),
            Reg16::DE => self.get_de(),
            Reg16::HL => self.get_hl(),
            Reg16::SP => self.sp,
        }
    }

    pub fn set_reg16(&mut self, reg: Reg16, value: u16) {
        match reg {
            Reg16::AF => self.set_af(value),
            Reg16::BC => self.set_bc(value),
            Reg16::DE => self.set_de(value),
            Reg16::HL => self.set_hl(value),
            Reg16::SP => self.sp = value,
        }
    }

    pub fn fetch_next_byte<B>(&mut self, bus: &B) -> u8
    where
        B: Bus,
    {
        let b = bus.read_byte(self.pc);

        self.pc = self.pc.wrapping_add(1);

        b
    }

    pub fn fetch_next_word<B>(&mut self, bus: &B) -> u16
    where
        B: Bus,
    {
        u16::from_le_bytes([self.fetch_next_byte(bus), self.fetch_next_byte(bus)])
    }

    pub fn fetch_next_instruction<B>(&mut self, bus: &B) -> Instruction
    where
        B: Bus,
    {
        let opcode = self.fetch_next_byte(bus);

        match opcode {
            0x00 => Instruction::Nop,

            0xd3 | 0xdb | 0xdd | 0xe3 | 0xe4 | 0xeb | 0xec | 0xed | 0xf4 | 0xfc | 0xfd => {
                Instruction::Illegal { opcode }
            }

            0x10 => Instruction::Stop {
                code: self.fetch_next_byte(bus),
            },

            0x76 => Instruction::Halt,
            0xf3 => Instruction::DisableInterrupts,
            0xfb => Instruction::EnableInterrupts,

            0x02 | 0x12 | 0x22 | 0x32 => Instruction::Load8 {
                dst: Operand8::from_ld_r16_opcode(opcode),
                src: Reg8::A.into(),
            },

            0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e | 0x36 | 0x3e => Instruction::Load8 {
                dst: Operand8::from_opcode(opcode),
                src: self.fetch_next_byte(bus).into(),
            },

            0x0a | 0x1a | 0x2a | 0x3a => Instruction::Load8 {
                dst: Reg8::A.into(),
                src: Operand8::from_ld_r16_opcode(opcode),
            },

            0x40..0x76 | 0x77..0x80 => Instruction::Load8 {
                dst: Operand8::from_opcode(opcode >> 3),
                src: Operand8::from_opcode(opcode),
            },

            0xea => Instruction::Load8 {
                dst: Operand8::Memory(self.fetch_next_word(bus).into()),
                src: Reg8::A.into(),
            },

            0xfa => Instruction::Load8 {
                dst: Reg8::A.into(),
                src: Operand8::Memory(self.fetch_next_word(bus).into()),
            },

            0xc6 => Instruction::Add {
                src: self.fetch_next_byte(bus).into(),
            },

            0x80..=0x87 => Instruction::Add {
                src: Operand8::from_opcode(opcode),
            },

            0xce => Instruction::Adc {
                src: self.fetch_next_byte(bus).into(),
            },

            0x88..=0x8f => Instruction::Adc {
                src: Operand8::from_opcode(opcode),
            },

            0xd6 => Instruction::Sub {
                src: self.fetch_next_byte(bus).into(),
            },

            0x90..=0x97 => Instruction::Sub {
                src: Operand8::from_opcode(opcode),
            },

            0xde => Instruction::Sbc {
                src: self.fetch_next_byte(bus).into(),
            },

            0x98..=0x9f => Instruction::Sbc {
                src: Operand8::from_opcode(opcode),
            },

            0xe6 => Instruction::And {
                src: self.fetch_next_byte(bus).into(),
            },

            0xa0..=0xa7 => Instruction::And {
                src: Operand8::from_opcode(opcode),
            },

            0xee => Instruction::Xor {
                src: self.fetch_next_byte(bus).into(),
            },

            0xa8..=0xaf => Instruction::Xor {
                src: Operand8::from_opcode(opcode),
            },

            0xf6 => Instruction::Or {
                src: self.fetch_next_byte(bus).into(),
            },

            0xb0..=0xb7 => Instruction::Or {
                src: Operand8::from_opcode(opcode),
            },

            0xfe => Instruction::Cp {
                src: self.fetch_next_byte(bus).into(),
            },

            0xb8..=0xbf => Instruction::Cp {
                src: Operand8::from_opcode(opcode),
            },

            0x18 => Instruction::JumpRelative {
                offset: self.fetch_next_byte(bus) as i8,
                condition: None,
            },
            0x20 | 0x28 | 0x30 | 0x38 => Instruction::JumpRelative {
                offset: self.fetch_next_byte(bus) as i8,
                condition: Some(JumpCondition::from_opcode(opcode)),
            },

            0xc9 => Instruction::Ret { condition: None },
            0xc0 | 0xc8 | 0xd0 | 0xd8 => Instruction::Ret {
                condition: Some(JumpCondition::from_opcode(opcode)),
            },

            0xc3 => Instruction::JumpAbsolute {
                address: self.fetch_next_word(bus),
                condition: None,
            },

            0xc2 | 0xca | 0xd2 | 0xda => Instruction::JumpAbsolute {
                address: self.fetch_next_word(bus),
                condition: Some(JumpCondition::from_opcode(opcode)),
            },

            0xe9 => Instruction::JumpHL,

            0xcd => Instruction::Call {
                address: self.fetch_next_word(bus),
                condition: None,
            },

            0xc4 | 0xcc | 0xd4 | 0xdc => Instruction::Call {
                address: self.fetch_next_word(bus),
                condition: Some(JumpCondition::from_opcode(opcode)),
            },

            0xd9 => Instruction::Reti,

            0xc1 | 0xd1 | 0xe1 | 0xf1 => Instruction::Pop {
                dst: Reg16::from_stack_opcode(opcode),
            },
            0xc5 | 0xd5 | 0xe5 | 0xf5 => Instruction::Push {
                src: Reg16::from_stack_opcode(opcode),
            },

            0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => {
                let vector = (opcode & 0b0011_1000) >> 3;

                Instruction::Rst { vector }
            }

            0x07 => Instruction::Rlca,
            0x0f => Instruction::Rrca,
            0x17 => Instruction::Rla,
            0x1f => Instruction::Rra,

            0x03 | 0x13 | 0x23 | 0x33 => Instruction::Inc16 {
                reg: Reg16::from_regular_opcode(opcode),
            },

            0x0b | 0x1b | 0x2b | 0x3b => Instruction::Dec16 {
                reg: Reg16::from_regular_opcode(opcode),
            },

            0x01 | 0x11 | 0x21 | 0x31 => Instruction::Load16 {
                dst: Reg16::from_regular_opcode(opcode).into(),
                src: Operand16::Immediate(self.fetch_next_word(bus)),
            },

            0xf8 => Instruction::LoadSPOffset {
                offset: self.fetch_next_byte(bus) as _,
            },

            0xf9 => Instruction::Load16 {
                dst: Reg16::SP.into(),
                src: Reg16::HL.into(),
            },

            0x04 | 0x0c | 0x14 | 0x1c | 0x24 | 0x2c | 0x34 | 0x3c => Instruction::Inc8 {
                reg: Operand8::from_opcode(opcode),
            },

            0x05 | 0x0d | 0x15 | 0x1d | 0x25 | 0x2d | 0x35 | 0x3d => Instruction::Dec8 {
                reg: Operand8::from_opcode(opcode),
            },

            0x27 => Instruction::Daa,
            0x2f => Instruction::Cpl,
            0x37 => Instruction::Scf,
            0x3f => Instruction::Ccf,

            0x09 | 0x19 | 0x29 | 0x39 => Instruction::Add16 {
                src: Reg16::from_regular_opcode(opcode),
            },

            0xe8 => Instruction::AddSP {
                offset: self.fetch_next_byte(bus) as i8,
            },

            0x08 => Instruction::Load16 {
                dst: Operand16::Absolute(self.fetch_next_word(bus)),
                src: Reg16::SP.into(),
            },

            0xe0 => Instruction::Load8 {
                dst: Address::HighImmediate(self.fetch_next_byte(bus)).into(),
                src: Reg8::A.into(),
            },

            0xf0 => Instruction::Load8 {
                dst: Reg8::A.into(),
                src: Address::HighImmediate(self.fetch_next_byte(bus)).into(),
            },

            0xe2 => Instruction::Load8 {
                dst: Address::HighC.into(),
                src: Reg8::A.into(),
            },

            0xf2 => Instruction::Load8 {
                dst: Reg8::A.into(),
                src: Address::HighC.into(),
            },

            0xcb => self.fetch_next_prefixed_instruction(bus),
        }
    }

    pub fn fetch_next_prefixed_instruction<B>(&mut self, bus: &B) -> Instruction
    where
        B: Bus,
    {
        let opcode = self.fetch_next_byte(bus);
        let src = Operand8::from_opcode(opcode);
        let bit = (opcode & 0b0011_1000) >> 3;

        match opcode {
            0x00..=0x07 => Instruction::Rlc { src },
            0x08..=0x0f => Instruction::Rrc { src },
            0x10..=0x17 => Instruction::Rl { src },
            0x18..=0x1f => Instruction::Rr { src },
            0x20..=0x27 => Instruction::Sla { src },
            0x28..=0x2f => Instruction::Sra { src },
            0x30..=0x37 => Instruction::Swap { src },
            0x38..=0x3f => Instruction::Srl { src },
            0x40..=0x7f => Instruction::Bit { bit, src },
            0x80..=0xbf => Instruction::Res { bit, src },
            0xc0..=0xff => Instruction::Set { bit, src },
        }
    }

    pub fn rl(src: u8, carry: bool) -> (u8, bool) {
        let carry_out = if carry { 0b0000_0001 } else { 0b0000_0000 };

        (src << 1 | carry_out, src & 0b1000_0000 == 0b1000_0000)
    }

    pub fn rr(src: u8, carry: bool) -> (u8, bool) {
        let carry_out = if carry { 0b1000_0000 } else { 0b0000_0000 };

        (carry_out | src >> 1, src & 0b0000_0001 == 0b0000_0001)
    }

    pub fn rlc(src: u8) -> (u8, bool) {
        (src.rotate_left(1), src & 0b1000_0000 == 0b1000_0000)
    }

    pub fn rrc(src: u8) -> (u8, bool) {
        (src.rotate_right(1), src & 0b0000_0001 == 0b0000_0001)
    }

    pub fn sla(src: u8) -> (u8, bool) {
        let carry_out = src & 0b1000_0000 == 0b1000_0000;

        (src << 1, carry_out)
    }

    pub fn sra(src: u8) -> (u8, bool) {
        let carry_out = src & 0b0000_0001 == 0b0000_0001;

        (src >> 1 | src & 0b1000_0000, carry_out)
    }

    pub fn srl(src: u8) -> (u8, bool) {
        let carry_out = src & 0b0000_0001 == 0b0000_0001;

        (src >> 1, carry_out)
    }

    pub fn add(lhs: u8, rhs: u8) -> (u8, bool, bool) {
        let (res, carry) = lhs.overflowing_add(rhs);
        let half_carry = (lhs & 0x0f) + (rhs & 0x0f) > 0x0f;

        (res, half_carry, carry)
    }

    pub fn adc(lhs: u8, rhs: u8, carry: bool) -> (u8, bool, bool) {
        let (res, carry) = lhs.carrying_add(rhs, carry);
        let (res_half, _) = (lhs & 0x0f).carrying_add(rhs & 0x0f, carry);
        let half_carry = res_half > 0x0f;

        (res, half_carry, carry)
    }

    pub fn sub(lhs: u8, rhs: u8) -> (u8, bool, bool) {
        todo!()
    }

    pub fn sbc(lhs: u8, rhs: u8, carry: bool) -> (u8, bool, bool) {
        todo!()
    }

    pub fn add16(lhs: u16, rhs: u16) -> (u16, bool, bool) {
        todo!()
    }

    pub fn add16_signed(lhs: u16, rhs: i8) -> (u16, bool, bool) {
        todo!()
    }

    pub fn handle_instruction<B>(&mut self, bus: &mut B, instruction: Instruction)
    where
        B: Bus,
    {
        match instruction {
            Instruction::Nop => (),
            Instruction::Illegal { opcode } => panic!("illegal instruction: {opcode:#04x}"),
            Instruction::EnableInterrupts => {
                todo!("set IME = 1 AFTER next instruction")
            }
            Instruction::DisableInterrupts => {
                todo!("set IME = 0")
            }
            Instruction::Push { src } => {
                let value = self.fetch_word(bus, src.into());

                self.push(bus, value);
            }
            Instruction::Pop { dst } => {
                let value = self.pop(bus);

                self.store_word(bus, dst.into(), value);
            }
            Instruction::Call { address, condition } => {
                if self.check_jump_condition(condition) {
                    self.call(bus, address);
                }

                // XXX return proper M-cycles
            }
            Instruction::Ret { condition } => {
                if self.check_jump_condition(condition) {
                    self.ret(bus);
                }

                // XXX return proper M-cycles
            }
            Instruction::Reti => {
                self.ret(bus);
                todo!("set IME = 1 AFTER next instruction")
            },
            Instruction::JumpHL => {
                let address = self.get_hl();

                self.jump_absolute(address);
            }
            Instruction::JumpAbsolute { address, condition } => {
                if self.check_jump_condition(condition) {
                    self.jump_absolute(address);
                }

                // XXX return proper M-cycles
            }
            Instruction::JumpRelative { offset, condition } => {
                if self.check_jump_condition(condition) {
                    self.jump_relative(offset);
                }

                // XXX return proper M-cycles
            }
            Instruction::Inc16 { reg } => {
                let value = self.get_reg16(reg);

                self.set_reg16(reg, value.wrapping_add(1));
            }
            Instruction::Dec16 { reg } => {
                let value = self.get_reg16(reg);

                self.set_reg16(reg, value.wrapping_sub(1));
            }
            Instruction::Add16 { src } => {
                let lhs = self.get_hl();
                let rhs = self.get_reg16(src);
                let (res, half, carry) = Self::add16(lhs, rhs);

                self.store_word(bus, src.into(), res);
                self.update_flags(|flags| {
                    flags.sub = false;
                    flags.half = half;
                    flags.carry = carry;
                });
            }
            Instruction::AddSP { offset } => {
                let (res, half, carry) = Self::add16_signed(self.sp, offset);

                self.sp = res;
                self.update_flags(|flags| {
                    flags.zero = false;
                    flags.sub = false;
                    flags.half = half;
                    flags.carry = carry;
                });
            }
            Instruction::LoadSPOffset { offset } => {
                let (res, half, carry) = Self::add16_signed(self.sp, offset);

                self.set_hl(res);
                self.update_flags(|flags| {
                    flags.zero = false;
                    flags.sub = false;
                    flags.half = half;
                    flags.carry = carry;
                });
            }
            Instruction::Add { src } => {
                let lhs = self.a;
                let rhs = self.fetch_byte(bus, src);
                let (res, half, carry) = Self::add(lhs, rhs);

                self.a = res;
                self.update_flags(|flags| {
                    flags.zero = res == 0;
                    flags.sub = false;
                    flags.half = half;
                    flags.carry = carry;
                });
            }
            Instruction::Adc { src } => {
                let lhs = self.a;
                let rhs = self.fetch_byte(bus, src);
                let carry = self.get_flags().carry;
                let (res, half, carry) = Self::adc(lhs, rhs, carry);

                self.a = res;
                self.update_flags(|flags| {
                    flags.zero = res == 0;
                    flags.sub = false;
                    flags.half = half;
                    flags.carry = carry;
                });
            }
            Instruction::Sub { src } => {
                let lhs = self.a;
                let rhs = self.fetch_byte(bus, src);
                let (res, half, carry) = Self::sub(lhs, rhs);

                self.a = res;
                self.update_flags(|flags| {
                    flags.zero = res == 0;
                    flags.sub = true;
                    flags.half = half;
                    flags.carry = carry;
                });
            }
            Instruction::Sbc { src } => {
                let lhs = self.a;
                let rhs = self.fetch_byte(bus, src);
                let carry = self.get_flags().carry;
                let (res, half, carry) = Self::sbc(lhs, rhs, carry);

                self.a = res;
                self.update_flags(|flags| {
                    flags.zero = res == 0;
                    flags.sub = true;
                    flags.half = half;
                    flags.carry = carry;
                });
            }
            Instruction::And { src } => {
                let lhs = self.a;
                let rhs = self.fetch_byte(bus, src);
                let res = lhs & rhs;

                self.a = res;
                self.update_flags(|flags| {
                    flags.zero = res == 0;
                    flags.sub = false;
                    flags.half = true;
                    flags.carry = false;
                });
            }
            Instruction::Xor { src } => {
                let lhs = self.a;
                let rhs = self.fetch_byte(bus, src);
                let res = lhs ^ rhs;

                self.a = res;
                self.update_flags(|flags| {
                    flags.zero = res == 0;
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = false;
                });
            }
            Instruction::Or { src } => {
                let lhs = self.a;
                let rhs = self.fetch_byte(bus, src);
                let res = lhs | rhs;

                self.a = res;
                self.update_flags(|flags| {
                    flags.zero = res == 0;
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = false;
                });
            }
            Instruction::Cp { src } => {
                let lhs = self.a;
                let rhs = self.fetch_byte(bus, src);
                let (res, half, carry) = Self::sub(lhs, rhs);

                self.update_flags(|flags| {
                    flags.zero = res == 0;
                    flags.sub = true;
                    flags.half = half;
                    flags.carry = carry;
                });
            }
            Instruction::Inc8 { reg } => {
                let value = self.fetch_byte(bus, reg);
                let (res, half, _) = Self::add(value, 1);

                self.store_byte(bus, reg, res);
                self.update_flags(|flags| {
                    flags.zero = res == 0;
                    flags.sub = false;
                    flags.half = half;
                });
            }
            Instruction::Dec8 { reg } => {
                let value = self.fetch_byte(bus, reg);
                let (res, half, _) = Self::sub(value, 1);

                self.store_byte(bus, reg, res);
                self.update_flags(|flags| {
                    flags.zero = res == 0;
                    flags.sub = true;
                    flags.half = half;
                });
            }
            Instruction::Load8 { dst, src } => {
                let value = self.fetch_byte(bus, src);

                self.store_byte(bus, dst, value);
            }
            Instruction::Load16 { dst, src } => {
                let value = self.fetch_word(bus, src);

                self.store_word(bus, dst, value);
            }
            Instruction::Rst { vector } => {
                self.call(bus, Self::high_address(vector));
            }
            Instruction::Cpl => {
                self.a = !self.a;
                self.update_flags(|flags| {
                    flags.sub = true;
                    flags.half = true;
                });
            }
            Instruction::Scf => {
                self.update_flags(|flags| {
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = true;
                });
            }
            Instruction::Ccf => {
                self.update_flags(|flags| {
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = !flags.carry;
                });
            }
            Instruction::Rlca => {
                let (value, carry) = Self::rlc(self.a);

                self.a = value;
                self.update_flags(|flags| {
                    flags.zero = false;
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = carry;
                });
            }
            Instruction::Rlc { src } => {
                let input = self.fetch_byte(bus, src);
                let (value, carry) = Self::rlc(input);

                self.store_byte(bus, src, value);
                self.update_flags(|flags| {
                    flags.zero = value == 0;
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = carry;
                });
            }
            Instruction::Rla => {
                let (value, carry) = Self::rl(self.a, self.get_flags().carry);

                self.a = value;
                self.update_flags(|flags| {
                    flags.zero = false;
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = carry;
                });
            }
            Instruction::Rl { src } => {
                let input = self.fetch_byte(bus, src);
                let (value, carry) = Self::rl(input, self.get_flags().carry);

                self.store_byte(bus, src, value);
                self.update_flags(|flags| {
                    flags.zero = value == 0;
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = carry;
                });
            }
            Instruction::Rrca => {
                let (value, carry) = Self::rrc(self.a);

                self.a = value;
                self.update_flags(|flags| {
                    flags.zero = false;
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = carry;
                });
            }
            Instruction::Rrc { src } => {
                let input = self.fetch_byte(bus, src);
                let (value, carry) = Self::rrc(input);

                self.store_byte(bus, src, value);
                self.update_flags(|flags| {
                    flags.zero = value == 0;
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = carry;
                });
            }
            Instruction::Rra => {
                let (value, carry) = Self::rr(self.a, self.get_flags().carry);

                self.a = value;
                self.update_flags(|flags| {
                    flags.zero = false;
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = carry;
                });
            }
            Instruction::Rr { src } => {
                let input = self.fetch_byte(bus, src);
                let (value, carry) = Self::rr(input, self.get_flags().carry);

                self.store_byte(bus, src, value);
                self.update_flags(|flags| {
                    flags.zero = value == 0;
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = carry;
                });
            }
            Instruction::Sla { src } => {
                let input = self.fetch_byte(bus, src);
                let (value, carry) = Self::sla(input);

                self.store_byte(bus, src, value);
                self.update_flags(|flags| {
                    flags.zero = value == 0;
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = carry;
                });
            }
            Instruction::Sra { src } => {
                let input = self.fetch_byte(bus, src);
                let (value, carry) = Self::sra(input);

                self.store_byte(bus, src, value);
                self.update_flags(|flags| {
                    flags.zero = value == 0;
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = carry;
                });
            }
            Instruction::Srl { src } => {
                let input = self.fetch_byte(bus, src);
                let (value, carry) = Self::srl(input);

                self.store_byte(bus, src, value);
                self.update_flags(|flags| {
                    flags.zero = value == 0;
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = carry;
                });
            }
            Instruction::Swap { src } => {
                let input = self.fetch_byte(bus, src);
                let value = input << 4 | input >> 4;

                self.store_byte(bus, src, value);
                self.update_flags(|flags| {
                    flags.zero = value == 0;
                    flags.sub = false;
                    flags.half = false;
                    flags.carry = false;
                });
            }
            Instruction::Bit { bit, src } => {
                let input = self.fetch_byte(bus, src);

                self.update_flags(|flags| {
                    flags.zero = input & (1 << bit) == 0;
                    flags.sub = false;
                    flags.half = true;
                })
            }
            Instruction::Res { bit, src } => {
                let input = self.fetch_byte(bus, src);
                let value = input & !(1 << bit);

                self.store_byte(bus, src, value);
            }
            Instruction::Set { bit, src } => {
                let input = self.fetch_byte(bus, src);
                let value = input | (1 << bit);

                self.store_byte(bus, src, value);
            }
            Instruction::Stop { .. } => todo!("stop"),
            Instruction::Halt => todo!("halt"),
            Instruction::Daa => todo!("daa"),
        }
    }

    fn high_address(off: u8) -> u16 {
        u16::wrapping_add(0xff00, off.into())
    }

    fn get_memory(&mut self, mem: Address) -> u16 {
        match mem {
            Address::Register(reg) => self.get_reg16(reg),
            Address::HL(mode) => {
                let step = match mode {
                    HLMode::Increment => 1,
                    HLMode::Decrement => -1,
                };

                let hl = self.get_hl();

                self.set_hl(hl.wrapping_sub_signed(step));

                hl
            }
            Address::Immediate(addr) => addr,
            Address::HighImmediate(off) => Self::high_address(off),
            Address::HighC => Self::high_address(self.c),
        }
    }

    pub fn fetch_byte<B>(&mut self, bus: &B, src: Operand8) -> u8
    where
        B: Bus,
    {
        match src {
            Operand8::Register(reg) => self.get_reg8(reg),
            Operand8::Immediate(b) => b,
            Operand8::Memory(mem) => bus.read_byte(self.get_memory(mem)),
        }
    }

    pub fn store_byte<B>(&mut self, bus: &mut B, dst: Operand8, value: u8)
    where
        B: Bus,
    {
        match dst {
            Operand8::Register(reg) => self.set_reg8(reg, value),
            Operand8::Immediate(_) => unreachable!("writing to immediate value"),
            Operand8::Memory(mem) => {
                let address = self.get_memory(mem);

                bus.write_byte(address, value)
            }
        }
    }

    pub fn fetch_word<B>(&mut self, bus: &B, src: Operand16) -> u16
    where
        B: Bus,
    {
        match src {
            Operand16::Register(reg) => self.get_reg16(reg),
            Operand16::Immediate(w) => w,
            Operand16::Absolute(address) => bus.read_word(address),
        }
    }

    pub fn store_word<B>(&mut self, bus: &mut B, dst: Operand16, value: u16)
    where
        B: Bus,
    {
        match dst {
            Operand16::Register(reg) => self.set_reg16(reg, value),
            Operand16::Immediate(_) => unreachable!("writing to immediate value"),
            Operand16::Absolute(address) => bus.write_word(address, value),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl std::fmt::Display for Reg8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::A => write!(f, "A"),
            Self::B => write!(f, "B"),
            Self::C => write!(f, "C"),
            Self::D => write!(f, "D"),
            Self::E => write!(f, "E"),
            Self::H => write!(f, "H"),
            Self::L => write!(f, "L"),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

impl Reg16 {
    pub fn from_regular_opcode(opcode: u8) -> Self {
        match (opcode & 0b0011_0000) >> 4 {
            0b00 => Self::BC,
            0b01 => Self::DE,
            0b10 => Self::HL,
            0b11 => Self::SP,
            _ => unreachable!(),
        }
    }

    pub fn from_stack_opcode(opcode: u8) -> Self {
        match (opcode & 0b0011_0000) >> 4 {
            0b00 => Self::BC,
            0b01 => Self::DE,
            0b10 => Self::HL,
            0b11 => Self::AF,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for Reg16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::AF => write!(f, "AF"),
            Self::BC => write!(f, "BC"),
            Self::DE => write!(f, "DE"),
            Self::HL => write!(f, "HL"),
            Self::SP => write!(f, "SP"),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum HLMode {
    Increment,
    Decrement,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Address {
    Register(Reg16),
    HL(HLMode),
    Immediate(u16),
    HighImmediate(u8),
    HighC,
}

impl Address {
    pub fn cycles(&self) -> u8 {
        match self {
            Self::Register(_) => 0,
            Self::HL(_) => 0,
            Self::Immediate(_) => 2,
            Self::HighImmediate(_) => 1,
            Self::HighC => 0,
        }
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Register(reg) => write!(f, "{reg}"),
            Self::HL(HLMode::Increment) => write!(f, "{reg}+", reg = Reg16::HL),
            Self::HL(HLMode::Decrement) => write!(f, "{reg}-", reg = Reg16::HL),
            Self::Immediate(address) => write!(f, "{address:#06x}"),
            Self::HighImmediate(offset) => write!(f, "{offset:#04x}"),
            Self::HighC => write!(f, "{reg}", reg = Reg8::C),
        }
    }
}

impl From<Reg16> for Address {
    fn from(reg: Reg16) -> Self {
        Self::Register(reg)
    }
}

impl From<HLMode> for Address {
    fn from(mode: HLMode) -> Self {
        Self::HL(mode)
    }
}

impl From<u16> for Address {
    fn from(address: u16) -> Self {
        Self::Immediate(address)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Operand8 {
    Register(Reg8),
    Immediate(u8),
    Memory(Address),
}

impl Operand8 {
    pub fn from_opcode(opcode: u8) -> Self {
        match opcode & 0b0000_0111 {
            0b0000_0000 => Reg8::B.into(),
            0b0000_0001 => Reg8::C.into(),
            0b0000_0010 => Reg8::D.into(),
            0b0000_0011 => Reg8::E.into(),
            0b0000_0100 => Reg8::H.into(),
            0b0000_0101 => Reg8::L.into(),
            0b0000_0110 => Address::Register(Reg16::HL).into(),
            0b0000_0111 => Reg8::A.into(),
            _ => unreachable!(),
        }
    }

    pub fn from_ld_r16_opcode(opcode: u8) -> Self {
        match (opcode & 0b0011_0000) >> 3 {
            0b00 => Self::Memory(Reg16::BC.into()),
            0b01 => Self::Memory(Reg16::DE.into()),
            0b10 => Address::HL(HLMode::Increment).into(),
            0b11 => Address::HL(HLMode::Decrement).into(),
            _ => unreachable!(),
        }
    }

    pub fn cycles(&self) -> u8 {
        match self {
            Self::Register(_) => 0,
            Self::Immediate(_) => 1,
            Self::Memory(address) => 1 + address.cycles(),
        }
    }
}

impl From<Reg8> for Operand8 {
    fn from(reg: Reg8) -> Self {
        Self::Register(reg)
    }
}

impl From<u8> for Operand8 {
    fn from(b: u8) -> Self {
        Self::Immediate(b)
    }
}

impl From<Address> for Operand8 {
    fn from(address: Address) -> Self {
        Self::Memory(address)
    }
}

impl std::fmt::Display for Operand8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Register(reg) => write!(f, "{reg}"),
            Self::Immediate(b) => write!(f, "{b:#04x}"),
            Self::Memory(address) => write!(f, "[{address}]"),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Operand16 {
    Register(Reg16),
    Immediate(u16),
    Absolute(u16),
}

impl std::fmt::Display for Operand16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Register(reg) => write!(f, "{reg}"),
            Self::Immediate(addr) => write!(f, "{addr:#06x}"),
            Self::Absolute(addr) => write!(f, "[{addr:#06x}]"),
        }
    }
}

impl From<Reg16> for Operand16 {
    fn from(reg: Reg16) -> Self {
        Self::Register(reg)
    }
}

#[derive(Debug, PartialEq)]
pub enum JumpCondition {
    NZ,
    Z,
    NC,
    C,
}

impl JumpCondition {
    pub fn from_opcode(bits: u8) -> Self {
        match (bits & 0b0001_1000) >> 3 {
            0b00 => Self::NZ,
            0b01 => Self::Z,
            0b10 => Self::NC,
            0b11 => Self::C,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for JumpCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::NZ => write!(f, "NZ"),
            Self::Z => write!(f, "Z"),
            Self::NC => write!(f, "NC"),
            Self::C => write!(f, "C"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Nop,
    Illegal {
        opcode: u8,
    },
    Stop {
        code: u8,
    },
    Halt,
    EnableInterrupts,
    DisableInterrupts,
    Load8 {
        dst: Operand8,
        src: Operand8,
    },
    JumpRelative {
        offset: i8,
        condition: Option<JumpCondition>,
    },
    JumpAbsolute {
        address: u16,
        condition: Option<JumpCondition>,
    },
    JumpHL,
    Call {
        address: u16,
        condition: Option<JumpCondition>,
    },
    Ret {
        condition: Option<JumpCondition>,
    },
    Reti,
    Push {
        src: Reg16,
    },
    Pop {
        dst: Reg16,
    },
    Rst {
        vector: u8,
    },

    Rlca,
    Rrca,
    Rla,
    Rra,

    Inc16 {
        reg: Reg16,
    },
    Dec16 {
        reg: Reg16,
    },

    Load16 {
        dst: Operand16,
        src: Operand16,
    },

    Add {
        src: Operand8,
    },

    Adc {
        src: Operand8,
    },

    Sub {
        src: Operand8,
    },

    Sbc {
        src: Operand8,
    },

    And {
        src: Operand8,
    },

    Xor {
        src: Operand8,
    },

    Or {
        src: Operand8,
    },

    Cp {
        src: Operand8,
    },

    Rlc {
        src: Operand8,
    },

    Rrc {
        src: Operand8,
    },

    Rl {
        src: Operand8,
    },

    Rr {
        src: Operand8,
    },

    Sla {
        src: Operand8,
    },

    Sra {
        src: Operand8,
    },

    Swap {
        src: Operand8,
    },

    Srl {
        src: Operand8,
    },

    Bit {
        bit: u8,
        src: Operand8,
    },

    Res {
        bit: u8,
        src: Operand8,
    },

    Set {
        bit: u8,
        src: Operand8,
    },

    Inc8 {
        reg: Operand8,
    },
    Dec8 {
        reg: Operand8,
    },

    Daa,
    Cpl,
    Scf,
    Ccf,

    Add16 {
        src: Reg16,
    },
    AddSP {
        offset: i8,
    },
    LoadSPOffset {
        offset: i8,
    },
}

impl Instruction {
    pub fn cycles(&self) -> u8 {
        match self {
            Self::Nop => 1,
            Self::Illegal { .. } => 1,
            Self::Stop { .. } => 2,
            Self::Halt => 1,
            Self::Load8 { dst, src } => 1 + dst.cycles() + src.cycles(),
            Self::JumpRelative { .. } => 2,
            Self::JumpAbsolute { .. } => 3,
            Self::JumpHL => 1,
            Self::Call { .. } => 3,
            Self::Ret { condition: None } => 4,
            Self::Ret { condition: Some(_) } => 2,
            Self::Reti => 4,
            Self::Push { .. } => 4,
            Self::Pop { .. } => 3,
            Self::EnableInterrupts => 1,
            Self::DisableInterrupts => 1,
            Self::Rst { .. } => 4,
            Self::Rlca => 1,
            Self::Rrca => 1,
            Self::Rla => 1,
            Self::Rra => 1,
            Self::Daa => 1,
            Self::Cpl => 1,
            Self::Scf => 1,
            Self::Ccf => 1,
            Self::Add16 { .. } => 2,
            Self::AddSP { .. } => 4,
            Self::Load16 { .. } => todo!(),
            Self::LoadSPOffset { .. } => 3,
            Self::Inc8 { reg } => 1 + reg.cycles() * 2,
            Self::Dec8 { reg } => 1 + reg.cycles() * 2,
            Self::Inc16 { .. } => 2,
            Self::Dec16 { .. } => 2,
            Self::Rlc { src } => 2 + src.cycles() * 2,
            Self::Rrc { src } => 2 + src.cycles() * 2,
            Self::Rl { src } => 2 + src.cycles() * 2,
            Self::Rr { src } => 2 + src.cycles() * 2,
            Self::Sra { src } => 2 + src.cycles() * 2,
            Self::Sla { src } => 2 + src.cycles() * 2,
            Self::Swap { src } => 2 + src.cycles() * 2,
            Self::Srl { src } => 2 + src.cycles() * 2,
            Self::Bit { src, .. } => 2 + src.cycles(),
            Self::Res { src, .. } => 2 + src.cycles() * 2,
            Self::Set { src, .. } => 2 + src.cycles() * 2,
            Self::Add { src } => 1 + src.cycles(),
            Self::Adc { src } => 1 + src.cycles(),
            Self::Sub { src } => 1 + src.cycles(),
            Self::Sbc { src } => 1 + src.cycles(),
            Self::And { src } => 1 + src.cycles(),
            Self::Xor { src } => 1 + src.cycles(),
            Self::Or { src } => 1 + src.cycles(),
            Self::Cp { src } => 1 + src.cycles(),
        }
    }

    pub fn branch_cycles(&self) -> Option<u8> {
        match self {
            Self::JumpRelative { .. } => Some(1),
            Self::JumpAbsolute { .. } => Some(1),
            Self::Call { .. } => Some(3),
            Self::Ret { condition: None } => None,
            Self::Ret { condition: Some(_) } => Some(3),
            Self::JumpHL => None,
            Self::Nop => None,
            Self::Illegal { .. } => None,
            Self::Stop { .. } => None,
            Self::Halt => None,
            Self::Load8 { .. } => None,
            Self::Reti => None,
            Self::Push { .. } => None,
            Self::Pop { .. } => None,
            Self::EnableInterrupts => None,
            Self::DisableInterrupts => None,
            Self::Rst { .. } => None,
            Self::Rlca => None,
            Self::Rrca => None,
            Self::Rla => None,
            Self::Rra => None,
            Self::Daa => None,
            Self::Cpl => None,
            Self::Scf => None,
            Self::Ccf => None,
            Self::Add16 { .. } => None,
            Self::AddSP { .. } => None,
            Self::Load16 { .. } => None,
            Self::LoadSPOffset { .. } => None,
            Self::Inc8 { .. } => None,
            Self::Dec8 { .. } => None,
            Self::Inc16 { .. } => None,
            Self::Dec16 { .. } => None,
            Self::Rlc { .. } => None,
            Self::Rrc { .. } => None,
            Self::Rl { .. } => None,
            Self::Rr { .. } => None,
            Self::Sra { .. } => None,
            Self::Sla { .. } => None,
            Self::Swap { .. } => None,
            Self::Srl { .. } => None,
            Self::Bit { .. } => None,
            Self::Res { .. } => None,
            Self::Set { .. } => None,
            Self::Add { .. } => None,
            Self::Adc { .. } => None,
            Self::Sub { .. } => None,
            Self::Sbc { .. } => None,
            Self::And { .. } => None,
            Self::Xor { .. } => None,
            Self::Or { .. } => None,
            Self::Cp { .. } => None,
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Nop => write!(f, "NOP"),

            Self::Illegal { opcode } => write!(f, "ILLEGAL_{opcode:02X}"),

            Self::Stop { code } => write!(f, "STOP {code:#04x}"),

            Self::Halt => write!(f, "HALT"),

            Self::EnableInterrupts => write!(f, "EI"),

            Self::DisableInterrupts => write!(f, "DI"),

            Self::Load8 { dst, src } => write!(f, "LD {dst}, {src}"),

            Self::JumpRelative {
                offset,
                condition: None,
            } => write!(f, "JR {offset}"),

            Self::JumpRelative {
                offset,
                condition: Some(condition),
            } => write!(f, "JR {condition}, {offset}"),

            Self::JumpAbsolute {
                address,
                condition: None,
            } => write!(f, "JP {address:#06x}"),

            Self::JumpAbsolute {
                address,
                condition: Some(condition),
            } => write!(f, "JP {condition}, {address:#06x}"),

            Self::JumpHL => write!(f, "JP {reg}", reg = Reg16::HL),

            Self::Call {
                address,
                condition: None,
            } => write!(f, "CALL {address:#06x}"),

            Self::Call {
                address,
                condition: Some(condition),
            } => write!(f, "CALL {condition}, {address:#06x}"),

            Self::Ret { condition: None } => write!(f, "RET"),

            Self::Ret {
                condition: Some(condition),
            } => write!(f, "RET {condition}"),

            Self::Reti => write!(f, "RETI"),

            Self::Push { src } => write!(f, "PUSH {src}"),

            Self::Pop { dst } => write!(f, "POP {dst}"),

            Self::Rst { vector } => write!(f, "RST ${vector:02x}"),

            Self::Rlca => write!(f, "RLCA"),
            Self::Rrca => write!(f, "RRCA"),
            Self::Rla => write!(f, "RLA"),
            Self::Rra => write!(f, "RRA"),

            Self::Inc16 { reg } => write!(f, "INC {reg}"),
            Self::Dec16 { reg } => write!(f, "DEC {reg}"),

            Self::Load16 { dst, src } => write!(f, "LD {dst}, {src}"),

            Self::Add { src } => write!(f, "ADD {dst}, {src}", dst = Reg8::A),
            Self::Adc { src } => write!(f, "ADC {dst}, {src}", dst = Reg8::A),
            Self::Sub { src } => write!(f, "SUB {dst}, {src}", dst = Reg8::A),
            Self::Sbc { src } => write!(f, "SBC {dst}, {src}", dst = Reg8::A),
            Self::And { src } => write!(f, "AND {dst}, {src}", dst = Reg8::A),
            Self::Xor { src } => write!(f, "XOR {dst}, {src}", dst = Reg8::A),
            Self::Or { src } => write!(f, "OR {dst}, {src}", dst = Reg8::A),
            Self::Cp { src } => write!(f, "CP {dst}, {src}", dst = Reg8::A),

            Self::Rlc { src } => write!(f, "RLC {src}"),
            Self::Rrc { src } => write!(f, "RRC {src}"),
            Self::Rl { src } => write!(f, "RL {src}"),
            Self::Rr { src } => write!(f, "RR {src}"),
            Self::Sla { src } => write!(f, "SLA {src}"),
            Self::Sra { src } => write!(f, "SRA {src}"),
            Self::Swap { src } => write!(f, "SWAP {src}"),
            Self::Srl { src } => write!(f, "SRL {src}"),
            Self::Bit { bit, src } => write!(f, "BIT {bit}, {src}"),
            Self::Res { bit, src } => write!(f, "RES {bit}, {src}"),
            Self::Set { bit, src } => write!(f, "SET {bit}, {src}"),

            Self::Inc8 { reg } => write!(f, "INC {reg}"),
            Self::Dec8 { reg } => write!(f, "DEC {reg}"),

            Self::Daa => write!(f, "DAA"),
            Self::Cpl => write!(f, "CPL"),
            Self::Scf => write!(f, "SCF"),
            Self::Ccf => write!(f, "CCF"),

            Self::Add16 { src } => write!(f, "ADD {dst}, {src}", dst = Reg16::HL),
            Self::AddSP { offset } => write!(f, "ADD {dst}, {offset}", dst = Reg16::SP),

            Self::LoadSPOffset { offset } => {
                let sign = if *offset < 0 { '-' } else { '+' };
                let offset = offset.unsigned_abs();

                write!(
                    f,
                    "SP {dst}, {src} {sign} {offset}",
                    dst = Reg16::HL,
                    src = Reg16::SP
                )
            }
        }
    }
}

#[cfg(test)]
#[path = "cpu_test.rs"]
mod tests;
