use crate::bus::Bus;

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

    pub fn modify_flags<F>(&mut self, f: F)
    where
        F: FnOnce(&mut CpuFlags),
    {
        let mut flags = self.get_flags();

        f(&mut flags);

        self.set_flags(flags);
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

            0x10 => {
                let code = self.fetch_next_byte(bus);

                Instruction::Stop { code }
            }

            0x76 => Instruction::Halt,
            0xf3 => Instruction::DisableInterrupts,
            0xfb => Instruction::EnableInterrupts,

            0x02 | 0x12 | 0x22 | 0x32 => {
                let dst = match opcode {
                    0x02 => Dst8::AtReg16(Reg16::BC),
                    0x12 => Dst8::AtReg16(Reg16::DE),
                    0x22 => Dst8::AtHLI,
                    0x32 => Dst8::AtHLD,
                    _ => unreachable!(),
                };

                Instruction::Load8 {
                    src: Src8::Reg8(Reg8::A),
                    dst,
                }
            }

            0x06 | 0x16 | 0x26 | 0x36 => {
                let dst = match opcode {
                    0x06 => Dst8::Reg8(Reg8::B),
                    0x16 => Dst8::Reg8(Reg8::D),
                    0x26 => Dst8::Reg8(Reg8::H),
                    0x36 => Dst8::AtReg16(Reg16::HL),
                    _ => unreachable!(),
                };
                let src = Src8::Imm8(self.fetch_next_byte(bus));

                Instruction::Load8 { src, dst }
            }

            0x0a | 0x1a | 0x2a | 0x3a => {
                let src = match opcode {
                    0x0a => Src8::AtReg16(Reg16::BC),
                    0x1a => Src8::AtReg16(Reg16::DE),
                    0x2a => Src8::AtHLI,
                    0x3a => Src8::AtHLD,
                    _ => unreachable!(),
                };

                Instruction::Load8 {
                    src,
                    dst: Dst8::Reg8(Reg8::A),
                }
            }

            0x0e | 0x1e | 0x2e | 0x3e => {
                let dst = match opcode {
                    0x0e => Dst8::Reg8(Reg8::C),
                    0x1e => Dst8::Reg8(Reg8::E),
                    0x2e => Dst8::Reg8(Reg8::L),
                    0x3e => Dst8::Reg8(Reg8::A),
                    _ => unreachable!(),
                };
                let src = Src8::Imm8(self.fetch_next_byte(bus));

                Instruction::Load8 { src, dst }
            }

            0x40..0x76 | 0x77..0x80 => {
                let src = match opcode & 0b0000_0111 {
                    0b0000_0000 => Src8::Reg8(Reg8::B),
                    0b0000_0001 => Src8::Reg8(Reg8::C),
                    0b0000_0010 => Src8::Reg8(Reg8::D),
                    0b0000_0011 => Src8::Reg8(Reg8::E),
                    0b0000_0100 => Src8::Reg8(Reg8::H),
                    0b0000_0101 => Src8::Reg8(Reg8::L),
                    0b0000_0110 => Src8::AtReg16(Reg16::HL),
                    0b0000_0111 => Src8::Reg8(Reg8::A),
                    _ => unreachable!(),
                };

                let dst = match (opcode & 0b0011_1000) >> 3 {
                    0b0000_0000 => Dst8::Reg8(Reg8::B),
                    0b0000_0001 => Dst8::Reg8(Reg8::C),
                    0b0000_0010 => Dst8::Reg8(Reg8::D),
                    0b0000_0011 => Dst8::Reg8(Reg8::E),
                    0b0000_0100 => Dst8::Reg8(Reg8::H),
                    0b0000_0101 => Dst8::Reg8(Reg8::L),
                    0b0000_0110 => Dst8::AtReg16(Reg16::HL),
                    0b0000_0111 => Dst8::Reg8(Reg8::A),
                    _ => unreachable!(),
                };

                Instruction::Load8 { src, dst }
            }

            0xea | 0xfa => {
                let address = self.fetch_next_word(bus);

                if opcode == 0xfa {
                    Instruction::Load8 {
                        dst: Dst8::AtImm16(address),
                        src: Src8::Reg8(Reg8::A),
                    }
                } else {
                    Instruction::Load8 {
                        dst: Dst8::Reg8(Reg8::A),
                        src: Src8::AtImm16(address),
                    }
                }
            }

            0xc6 => Instruction::Add {
                src: Arith8::Imm8(self.fetch_next_byte(bus)),
            },

            0x80..=0x87 => {
                let src = match opcode & 0b0000_0111 {
                    0b0000_0000 => Arith8::Reg8(Reg8::B),
                    0b0000_0001 => Arith8::Reg8(Reg8::C),
                    0b0000_0010 => Arith8::Reg8(Reg8::D),
                    0b0000_0011 => Arith8::Reg8(Reg8::E),
                    0b0000_0100 => Arith8::Reg8(Reg8::H),
                    0b0000_0101 => Arith8::Reg8(Reg8::L),
                    0b0000_0110 => Arith8::AtHL,
                    0b0000_0111 => Arith8::Reg8(Reg8::A),
                    _ => unreachable!(),
                };

                Instruction::Add { src }
            }

            0xce => Instruction::Adc {
                src: Arith8::Imm8(self.fetch_next_byte(bus)),
            },

            0x88..=0x8f => {
                let src = match opcode & 0b0000_0111 {
                    0b0000_0000 => Arith8::Reg8(Reg8::B),
                    0b0000_0001 => Arith8::Reg8(Reg8::C),
                    0b0000_0010 => Arith8::Reg8(Reg8::D),
                    0b0000_0011 => Arith8::Reg8(Reg8::E),
                    0b0000_0100 => Arith8::Reg8(Reg8::H),
                    0b0000_0101 => Arith8::Reg8(Reg8::L),
                    0b0000_0110 => Arith8::AtHL,
                    0b0000_0111 => Arith8::Reg8(Reg8::A),
                    _ => unreachable!(),
                };

                Instruction::Adc { src }
            }

            0xd6 => Instruction::Sub {
                src: Arith8::Imm8(self.fetch_next_byte(bus)),
            },

            0x90..=0x97 => {
                let src = match opcode & 0b0000_0111 {
                    0b0000_0000 => Arith8::Reg8(Reg8::B),
                    0b0000_0001 => Arith8::Reg8(Reg8::C),
                    0b0000_0010 => Arith8::Reg8(Reg8::D),
                    0b0000_0011 => Arith8::Reg8(Reg8::E),
                    0b0000_0100 => Arith8::Reg8(Reg8::H),
                    0b0000_0101 => Arith8::Reg8(Reg8::L),
                    0b0000_0110 => Arith8::AtHL,
                    0b0000_0111 => Arith8::Reg8(Reg8::A),
                    _ => unreachable!(),
                };

                Instruction::Sub { src }
            }

            0xde => Instruction::Sbc {
                src: Arith8::Imm8(self.fetch_next_byte(bus)),
            },

            0x98..=0x9f => {
                let src = match opcode & 0b0000_0111 {
                    0b0000_0000 => Arith8::Reg8(Reg8::B),
                    0b0000_0001 => Arith8::Reg8(Reg8::C),
                    0b0000_0010 => Arith8::Reg8(Reg8::D),
                    0b0000_0011 => Arith8::Reg8(Reg8::E),
                    0b0000_0100 => Arith8::Reg8(Reg8::H),
                    0b0000_0101 => Arith8::Reg8(Reg8::L),
                    0b0000_0110 => Arith8::AtHL,
                    0b0000_0111 => Arith8::Reg8(Reg8::A),
                    _ => unreachable!(),
                };

                Instruction::Sbc { src }
            }

            0xe6 => Instruction::And {
                src: Arith8::Imm8(self.fetch_next_byte(bus)),
            },

            0xa0..=0xa7 => {
                let src = match opcode & 0b0000_0111 {
                    0b0000_0000 => Arith8::Reg8(Reg8::B),
                    0b0000_0001 => Arith8::Reg8(Reg8::C),
                    0b0000_0010 => Arith8::Reg8(Reg8::D),
                    0b0000_0011 => Arith8::Reg8(Reg8::E),
                    0b0000_0100 => Arith8::Reg8(Reg8::H),
                    0b0000_0101 => Arith8::Reg8(Reg8::L),
                    0b0000_0110 => Arith8::AtHL,
                    0b0000_0111 => Arith8::Reg8(Reg8::A),
                    _ => unreachable!(),
                };

                Instruction::And { src }
            }

            0xee => Instruction::Xor {
                src: Arith8::Imm8(self.fetch_next_byte(bus)),
            },

            0xa8..=0xaf => {
                let src = match opcode & 0b0000_0111 {
                    0b0000_0000 => Arith8::Reg8(Reg8::B),
                    0b0000_0001 => Arith8::Reg8(Reg8::C),
                    0b0000_0010 => Arith8::Reg8(Reg8::D),
                    0b0000_0011 => Arith8::Reg8(Reg8::E),
                    0b0000_0100 => Arith8::Reg8(Reg8::H),
                    0b0000_0101 => Arith8::Reg8(Reg8::L),
                    0b0000_0110 => Arith8::AtHL,
                    0b0000_0111 => Arith8::Reg8(Reg8::A),
                    _ => unreachable!(),
                };

                Instruction::Xor { src }
            }

            0xf6 => Instruction::Or {
                src: Arith8::Imm8(self.fetch_next_byte(bus)),
            },

            0xb0..=0xb7 => {
                let src = match opcode & 0b0000_0111 {
                    0b0000_0000 => Arith8::Reg8(Reg8::B),
                    0b0000_0001 => Arith8::Reg8(Reg8::C),
                    0b0000_0010 => Arith8::Reg8(Reg8::D),
                    0b0000_0011 => Arith8::Reg8(Reg8::E),
                    0b0000_0100 => Arith8::Reg8(Reg8::H),
                    0b0000_0101 => Arith8::Reg8(Reg8::L),
                    0b0000_0110 => Arith8::AtHL,
                    0b0000_0111 => Arith8::Reg8(Reg8::A),
                    _ => unreachable!(),
                };

                Instruction::Or { src }
            }

            0xfe => Instruction::Cp {
                src: Arith8::Imm8(self.fetch_next_byte(bus)),
            },

            0xb8..=0xbf => {
                let src = match opcode & 0b0000_0111 {
                    0b0000_0000 => Arith8::Reg8(Reg8::B),
                    0b0000_0001 => Arith8::Reg8(Reg8::C),
                    0b0000_0010 => Arith8::Reg8(Reg8::D),
                    0b0000_0011 => Arith8::Reg8(Reg8::E),
                    0b0000_0100 => Arith8::Reg8(Reg8::H),
                    0b0000_0101 => Arith8::Reg8(Reg8::L),
                    0b0000_0110 => Arith8::AtHL,
                    0b0000_0111 => Arith8::Reg8(Reg8::A),
                    _ => unreachable!(),
                };

                Instruction::Cp { src }
            }

            0x18 => Instruction::JumpRelative {
                offset: self.fetch_next_byte(bus) as i8,
                condition: None,
            },
            0x20 => Instruction::JumpRelative {
                offset: self.fetch_next_byte(bus) as i8,
                condition: Some(JumpCondition::NZ),
            },
            0x28 => Instruction::JumpRelative {
                offset: self.fetch_next_byte(bus) as i8,
                condition: Some(JumpCondition::Z),
            },
            0x30 => Instruction::JumpRelative {
                offset: self.fetch_next_byte(bus) as i8,
                condition: Some(JumpCondition::NC),
            },
            0x38 => Instruction::JumpRelative {
                offset: self.fetch_next_byte(bus) as i8,
                condition: Some(JumpCondition::C),
            },

            0xc9 => Instruction::Ret { condition: None },
            0xc0 => Instruction::Ret {
                condition: Some(JumpCondition::NZ),
            },
            0xc8 => Instruction::Ret {
                condition: Some(JumpCondition::Z),
            },
            0xd0 => Instruction::Ret {
                condition: Some(JumpCondition::NC),
            },
            0xd8 => Instruction::Ret {
                condition: Some(JumpCondition::C),
            },

            0xc3 => Instruction::JumpAbsolute {
                target: AbsoluteJumpTarget::Imm16(self.fetch_next_word(bus)),
                condition: None,
            },
            0xc2 => Instruction::JumpAbsolute {
                target: AbsoluteJumpTarget::Imm16(self.fetch_next_word(bus)),
                condition: Some(JumpCondition::NZ),
            },
            0xca => Instruction::JumpAbsolute {
                target: AbsoluteJumpTarget::Imm16(self.fetch_next_word(bus)),
                condition: Some(JumpCondition::Z),
            },
            0xd2 => Instruction::JumpAbsolute {
                target: AbsoluteJumpTarget::Imm16(self.fetch_next_word(bus)),
                condition: Some(JumpCondition::NC),
            },
            0xda => Instruction::JumpAbsolute {
                target: AbsoluteJumpTarget::Imm16(self.fetch_next_word(bus)),
                condition: Some(JumpCondition::C),
            },
            0xe9 => Instruction::JumpAbsolute {
                target: AbsoluteJumpTarget::HL,
                condition: None,
            },

            0xcd => Instruction::Call {
                address: self.fetch_next_word(bus),
                condition: None,
            },
            0xc4 => Instruction::Call {
                address: self.fetch_next_word(bus),
                condition: Some(JumpCondition::NZ),
            },
            0xcc => Instruction::Call {
                address: self.fetch_next_word(bus),
                condition: Some(JumpCondition::Z),
            },
            0xd4 => Instruction::Call {
                address: self.fetch_next_word(bus),
                condition: Some(JumpCondition::NC),
            },
            0xdc => Instruction::Call {
                address: self.fetch_next_word(bus),
                condition: Some(JumpCondition::C),
            },

            0xd9 => Instruction::Reti,

            0xc1 => Instruction::Pop { dst: Reg16::BC },
            0xd1 => Instruction::Pop { dst: Reg16::DE },
            0xe1 => Instruction::Pop { dst: Reg16::HL },
            0xf1 => Instruction::Pop { dst: Reg16::AF },

            0xc5 => Instruction::Push { src: Reg16::BC },
            0xd5 => Instruction::Push { src: Reg16::DE },
            0xe5 => Instruction::Push { src: Reg16::HL },
            0xf5 => Instruction::Push { src: Reg16::AF },

            0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => {
                let vector = (opcode & 0b0011_1000) >> 3;

                Instruction::Rst { vector }
            }

            0x07 => Instruction::Rlca,
            0x0f => Instruction::Rrca,
            0x17 => Instruction::Rla,
            0x1f => Instruction::Rra,

            0x03 => Instruction::Inc16 { reg: Reg16::BC },
            0x13 => Instruction::Inc16 { reg: Reg16::DE },
            0x23 => Instruction::Inc16 { reg: Reg16::HL },
            0x33 => Instruction::Inc16 { reg: Reg16::SP },

            0x0b => Instruction::Dec16 { reg: Reg16::BC },
            0x1b => Instruction::Dec16 { reg: Reg16::DE },
            0x2b => Instruction::Dec16 { reg: Reg16::HL },
            0x3b => Instruction::Dec16 { reg: Reg16::SP },

            0x01 => Instruction::Load16 {
                dst: Dst16::Reg16(Reg16::BC),
                src: Src16::Imm16(self.fetch_next_word(bus)),
            },

            0x11 => Instruction::Load16 {
                dst: Dst16::Reg16(Reg16::DE),
                src: Src16::Imm16(self.fetch_next_word(bus)),
            },

            0x21 => Instruction::Load16 {
                dst: Dst16::Reg16(Reg16::HL),
                src: Src16::Imm16(self.fetch_next_word(bus)),
            },

            0x31 => Instruction::Load16 {
                dst: Dst16::Reg16(Reg16::SP),
                src: Src16::Imm16(self.fetch_next_word(bus)),
            },

            0xf8 => Instruction::Load16 {
                dst: Dst16::Reg16(Reg16::HL),
                src: Src16::Reg16Offset {
                    reg: Reg16::SP,
                    offset: self.fetch_next_byte(bus) as i8,
                },
            },

            0xf9 => Instruction::Load16 {
                dst: Dst16::Reg16(Reg16::SP),
                src: Src16::Reg16(Reg16::HL),
            },

            0x04 | 0x0c | 0x14 | 0x1c | 0x24 | 0x2c | 0x34 | 0x3c => {
                let reg = match opcode & 0b0000_0111 {
                    0b0000_0000 => Arith8::Reg8(Reg8::B),
                    0b0000_0001 => Arith8::Reg8(Reg8::C),
                    0b0000_0010 => Arith8::Reg8(Reg8::D),
                    0b0000_0011 => Arith8::Reg8(Reg8::E),
                    0b0000_0100 => Arith8::Reg8(Reg8::H),
                    0b0000_0101 => Arith8::Reg8(Reg8::L),
                    0b0000_0110 => Arith8::AtHL,
                    0b0000_0111 => Arith8::Reg8(Reg8::A),
                    _ => unreachable!(),
                };

                Instruction::Inc8 { reg }
            }

            0x05 | 0x0d | 0x15 | 0x1d | 0x25 | 0x2d | 0x35 | 0x3d => {
                let reg = match opcode & 0b0000_0111 {
                    0b0000_0000 => Arith8::Reg8(Reg8::B),
                    0b0000_0001 => Arith8::Reg8(Reg8::C),
                    0b0000_0010 => Arith8::Reg8(Reg8::D),
                    0b0000_0011 => Arith8::Reg8(Reg8::E),
                    0b0000_0100 => Arith8::Reg8(Reg8::H),
                    0b0000_0101 => Arith8::Reg8(Reg8::L),
                    0b0000_0110 => Arith8::AtHL,
                    0b0000_0111 => Arith8::Reg8(Reg8::A),
                    _ => unreachable!(),
                };

                Instruction::Dec8 { reg }
            }

            0x27 => Instruction::Daa,
            0x2f => Instruction::Cpl,
            0x37 => Instruction::Scf,
            0x3f => Instruction::Ccf,

            0x09 => Instruction::Add16 { src: Reg16::BC },
            0x19 => Instruction::Add16 { src: Reg16::DE },
            0x29 => Instruction::Add16 { src: Reg16::HL },
            0x39 => Instruction::Add16 { src: Reg16::SP },
            0xe8 => Instruction::AddSP {
                offset: self.fetch_next_byte(bus) as i8,
            },

            0x08 => Instruction::Load16 {
                dst: Dst16::AtImm16(self.fetch_next_word(bus)),
                src: Src16::Reg16(Reg16::SP),
            },

            0xe0 => Instruction::LoadHigh {
                dst: OperandHigh::AtImm8(self.fetch_next_byte(bus)),
                src: OperandHigh::A,
            },
            0xf0 => Instruction::LoadHigh {
                dst: OperandHigh::A,
                src: OperandHigh::AtImm8(self.fetch_next_byte(bus)),
            },

            0xe2 => Instruction::LoadHigh {
                dst: OperandHigh::AtC,
                src: OperandHigh::A,
            },
            0xf2 => Instruction::LoadHigh {
                dst: OperandHigh::A,
                src: OperandHigh::AtC,
            },

            0xcb => self.fetch_next_prefixed_instruction(bus),
        }
    }

    pub fn fetch_next_prefixed_instruction<B>(&mut self, bus: &B) -> Instruction
    where
        B: Bus,
    {
        let opcode = self.fetch_next_byte(bus);
        let src = match opcode & 0b0000_0111 {
            0b0000_0000 => Arith8::Reg8(Reg8::B),
            0b0000_0001 => Arith8::Reg8(Reg8::C),
            0b0000_0010 => Arith8::Reg8(Reg8::D),
            0b0000_0011 => Arith8::Reg8(Reg8::E),
            0b0000_0100 => Arith8::Reg8(Reg8::H),
            0b0000_0101 => Arith8::Reg8(Reg8::L),
            0b0000_0110 => Arith8::AtHL,
            0b0000_0111 => Arith8::Reg8(Reg8::A),
            _ => unreachable!(),
        };
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

    pub fn handle_instruction<B>(&mut self, _: &mut B, _: Instruction) -> usize
    where
        B: Bus,
    {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
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

#[derive(Debug, PartialEq)]
pub enum Src8 {
    Reg8(Reg8),
    Imm8(u8),
    AtReg16(Reg16),
    AtImm16(u16),
    AtHLI,
    AtHLD,
}

impl std::fmt::Display for Src8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Reg8(reg) => write!(f, "{reg}"),
            Self::Imm8(b) => write!(f, "{b:#04x}"),
            Self::AtReg16(reg) => write!(f, "[{reg}]"),
            Self::AtImm16(addr) => write!(f, "[{addr:#06x}]"),
            Self::AtHLI => write!(f, "[{}+]", Reg16::HL),
            Self::AtHLD => write!(f, "[{}-]", Reg16::HL),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Dst8 {
    Reg8(Reg8),
    AtReg16(Reg16),
    AtImm16(u16),
    AtHLI,
    AtHLD,
}

impl std::fmt::Display for Dst8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Reg8(reg) => write!(f, "{reg}"),
            Self::AtReg16(reg) => write!(f, "[{reg}]"),
            Self::AtImm16(addr) => write!(f, "[{addr:#06x}]"),
            Self::AtHLI => write!(f, "[{}+]", Reg16::HL),
            Self::AtHLD => write!(f, "[{}-]", Reg16::HL),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum OperandHigh {
    A,
    AtC,
    AtImm8(u8),
}

impl std::fmt::Display for OperandHigh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::A => write!(f, "A"),
            Self::AtC => write!(f, "[{}]", Reg8::C),
            Self::AtImm8(b) => write!(f, "[{b:#04x}]"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Arith8 {
    Reg8(Reg8),
    AtHL,
    Imm8(u8),
}

impl std::fmt::Display for Arith8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Reg8(reg) => write!(f, "{reg}"),
            Self::AtHL => write!(f, "[{}]", Reg16::HL),
            Self::Imm8(b) => write!(f, "{b:#04x}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Dst16 {
    Reg16(Reg16),
    AtImm16(u16),
}

impl std::fmt::Display for Dst16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Reg16(reg) => write!(f, "{reg}"),
            Self::AtImm16(addr) => write!(f, "{addr:#06x}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Src16 {
    Reg16(Reg16),
    Imm16(u16),
    Reg16Offset { reg: Reg16, offset: i8 },
}

impl std::fmt::Display for Src16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Reg16(reg) => write!(f, "{reg}"),
            Self::Imm16(w) => write!(f, "{w:#06x}"),
            Self::Reg16Offset { reg, offset } => {
                let sign = if *offset < 0 { '-' } else { '+' };

                let offset = offset.unsigned_abs();

                write!(f, "{reg} {sign} {offset}")
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum JumpCondition {
    NZ,
    Z,
    NC,
    C,
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
pub enum AbsoluteJumpTarget {
    Imm16(u16),
    HL,
}

impl std::fmt::Display for AbsoluteJumpTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Imm16(addr) => write!(f, "{addr:#06x}"),
            Self::HL => write!(f, "{}", Reg16::HL),
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
        dst: Dst8,
        src: Src8,
    },
    LoadHigh {
        dst: OperandHigh,
        src: OperandHigh,
    },
    JumpRelative {
        offset: i8,
        condition: Option<JumpCondition>,
    },
    JumpAbsolute {
        target: AbsoluteJumpTarget,
        condition: Option<JumpCondition>,
    },
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
        dst: Dst16,
        src: Src16,
    },

    Add {
        src: Arith8,
    },

    Adc {
        src: Arith8,
    },

    Sub {
        src: Arith8,
    },

    Sbc {
        src: Arith8,
    },

    And {
        src: Arith8,
    },

    Xor {
        src: Arith8,
    },

    Or {
        src: Arith8,
    },

    Cp {
        src: Arith8,
    },

    Rlc {
        src: Arith8,
    },

    Rrc {
        src: Arith8,
    },

    Rl {
        src: Arith8,
    },

    Rr {
        src: Arith8,
    },

    Sla {
        src: Arith8,
    },

    Sra {
        src: Arith8,
    },

    Swap {
        src: Arith8,
    },

    Srl {
        src: Arith8,
    },

    Bit {
        bit: u8,
        src: Arith8,
    },

    Res {
        bit: u8,
        src: Arith8,
    },

    Set {
        bit: u8,
        src: Arith8,
    },

    Inc8 {
        reg: Arith8,
    },
    Dec8 {
        reg: Arith8,
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

            Self::LoadHigh { dst, src } => write!(f, "LDH {dst}, {src}"),

            Self::JumpRelative {
                offset,
                condition: None,
            } => write!(f, "JR {offset}"),

            Self::JumpRelative {
                offset,
                condition: Some(condition),
            } => write!(f, "JR {condition}, {offset}"),

            Self::JumpAbsolute {
                target,
                condition: None,
            } => write!(f, "JP {target}"),

            Self::JumpAbsolute {
                target,
                condition: Some(condition),
            } => write!(f, "JP {condition}, {target}"),

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

            Self::Add { src } => write!(f, "ADD {}, {src}", Reg8::A),
            Self::Adc { src } => write!(f, "ADC {}, {src}", Reg8::A),
            Self::Sub { src } => write!(f, "SUB {}, {src}", Reg8::A),
            Self::Sbc { src } => write!(f, "SBC {}, {src}", Reg8::A),
            Self::And { src } => write!(f, "AND {}, {src}", Reg8::A),
            Self::Xor { src } => write!(f, "XOR {}, {src}", Reg8::A),
            Self::Or { src } => write!(f, "OR {}, {src}", Reg8::A),
            Self::Cp { src } => write!(f, "CP {}, {src}", Reg8::A),

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

            Self::Add16 { src } => write!(f, "ADD {}, {src}", Reg16::HL),
            Self::AddSP { offset } => write!(f, "ADD {}, {offset}", Reg16::SP),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBus {
        data: Vec<u8>,
    }

    impl Bus for TestBus {
        fn read_byte(&self, address: u16) -> u8 {
            self.data[address as usize]
        }

        fn write_byte(&mut self, address: u16, value: u8) {
            self.data[address as usize] = value
        }
    }

    #[test]
    fn decode_nop() {
        let mut cpu = Cpu::default();
        let mut bus = TestBus { data: vec![0x00] };

        let instruction = cpu.fetch_next_instruction(&mut bus);

        assert_eq!(instruction, Instruction::Nop);
    }

    #[test]
    fn decode_nop_multiple() {
        let mut cpu = Cpu::default();
        let mut bus = TestBus {
            data: vec![
                0x00, // NOP
                0x00, // NOP
                0x00, // NOP
            ],
        };

        assert_eq!(cpu.fetch_next_instruction(&mut bus), Instruction::Nop);
        assert_eq!(cpu.fetch_next_instruction(&mut bus), Instruction::Nop);
        assert_eq!(cpu.fetch_next_instruction(&mut bus), Instruction::Nop);
    }

    #[test]
    fn decode_mixed() {
        let mut cpu = Cpu::default();
        let mut bus = TestBus {
            data: vec![
                0x10, 0x00, // STOP 0x00
                0xfb, // EI
                0x00, // NOP
                0x10, 0x10, // STOP 0x10
                0xf3, // DI
            ],
        };

        assert_eq!(
            cpu.fetch_next_instruction(&mut bus),
            Instruction::Stop { code: 0x00 }
        );
        assert_eq!(
            cpu.fetch_next_instruction(&mut bus),
            Instruction::EnableInterrupts
        );
        assert_eq!(cpu.fetch_next_instruction(&mut bus), Instruction::Nop);
        assert_eq!(
            cpu.fetch_next_instruction(&mut bus),
            Instruction::Stop { code: 0x10 }
        );
        assert_eq!(
            cpu.fetch_next_instruction(&mut bus),
            Instruction::DisableInterrupts
        );
    }

    #[test]
    fn decode_ld8() {
        let mut cpu = Cpu::default();
        let mut bus = TestBus {
            data: vec![
                0x7f, // LD A, A
                0x7f, // LD A, A
                0x3e, 0x42, // LD A, 0x42
                0x22, // LD [HL+], A
            ],
        };

        assert_eq!(
            cpu.fetch_next_instruction(&mut bus),
            Instruction::Load8 {
                dst: Dst8::Reg8(Reg8::A),
                src: Src8::Reg8(Reg8::A),
            },
        );

        assert_eq!(
            cpu.fetch_next_instruction(&mut bus),
            Instruction::Load8 {
                dst: Dst8::Reg8(Reg8::A),
                src: Src8::Reg8(Reg8::A),
            },
        );

        assert_eq!(
            cpu.fetch_next_instruction(&mut bus),
            Instruction::Load8 {
                dst: Dst8::Reg8(Reg8::A),
                src: Src8::Imm8(0x42),
            },
        );

        assert_eq!(
            cpu.fetch_next_instruction(&mut bus),
            Instruction::Load8 {
                dst: Dst8::AtHLI,
                src: Src8::Reg8(Reg8::A),
            },
        );
    }
}
