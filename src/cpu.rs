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

    pub fn fetch_next_instruction<B>(&mut self, bus: &B) -> Instruction
    where
        B: Bus,
    {
        let opcode = self.fetch_next_byte(bus);

        match opcode {
            0x00 => Instruction::Nop,
            0xd3 | 0xdb | 0xe3 | 0xe4 | 0xeb | 0xec | 0xed | 0xf4 | 0xfc | 0xfd => {
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
            _ => todo!("unhandled instruction: {opcode:#04x}"),
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
    F,
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
            Self::F => write!(f, "F"),
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
    PC,
    SP,
}

impl std::fmt::Display for Reg16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::AF => write!(f, "AF"),
            Self::BC => write!(f, "BC"),
            Self::DE => write!(f, "DE"),
            Self::HL => write!(f, "HL"),
            Self::PC => write!(f, "PC"),
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
pub enum Instruction {
    Nop,
    Illegal { opcode: u8 },
    Stop { code: u8 },
    Halt,
    EnableInterrupts,
    DisableInterrupts,
    Load8 { dst: Dst8, src: Src8 },
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
