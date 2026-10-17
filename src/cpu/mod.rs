use crate::bus::{Bus, BusExt};

mod flags;
pub use flags::*;

mod operands;
pub use operands::*;

mod instructions;
pub use instructions::*;

mod decoder;
use decoder::Decoder;

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
    pub ime: bool,
    pub ime_pending: bool,
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
        let mut decoder = Decoder::new(self.pc);
        let (instruction, rc) = decoder.decode_instruction(bus);

        self.pc = self.pc.wrapping_add(rc);

        instruction
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

    pub fn handle_interrupts<B>(&mut self, bus: &mut B) -> u8
    where
        B: Bus,
    {
        let enable = bus.read_byte(0xffff);
        let flag = bus.read_byte(0xff0f);
        let pending = enable & flag & 0b0001_1111;

        let Some(interrupt) = pending.lowest_one() else {
            return 0;
        };

        // XXX wake from halt/stop

        if self.ime {
            self.call(bus, 0x40 + interrupt as u16 * 8);
            5
        } else {
            0
        }
    }

    fn handle_instruction_inner<B>(&mut self, bus: &mut B, instruction: Instruction) -> u8
    where
        B: Bus,
    {
        let mut cycles = instruction.cycles();
        let branch_cycles = instruction.branch_cycles();

        match instruction {
            Instruction::Nop => (),
            Instruction::Illegal { opcode } => panic!("illegal instruction: {opcode:#04x}"),
            Instruction::EnableInterrupts => {
                self.ime_pending = true;
            }
            Instruction::DisableInterrupts => {
                self.ime = false;
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

                    cycles += branch_cycles;
                }
            }
            Instruction::Ret { condition } => {
                if self.check_jump_condition(condition) {
                    self.ret(bus);

                    cycles += branch_cycles;
                }
            }
            Instruction::Reti => {
                self.ret(bus);
                self.ime_pending = true;
            }
            Instruction::JumpHL => {
                let address = self.get_hl();

                self.jump_absolute(address);
            }
            Instruction::JumpAbsolute { address, condition } => {
                if self.check_jump_condition(condition) {
                    self.jump_absolute(address);

                    cycles += branch_cycles;
                }
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

        cycles
    }

    pub fn handle_instruction<B>(&mut self, bus: &mut B, instruction: Instruction) -> u8
    where
        B: Bus,
    {
        // XXX move to executor

        let ime_pending = self.ime_pending;
        let cycles = self.handle_instruction_inner(bus, instruction);

        if ime_pending {
            // XXX wrong if halted

            self.ime = true;
            self.ime_pending = false;
        }

        cycles
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
