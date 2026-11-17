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
        match (opcode & 0b0011_0000) >> 4 {
            0b00 => Self::Memory(Reg16::BC.into()),
            0b01 => Self::Memory(Reg16::DE.into()),
            0b10 => Address::HL(HLMode::Increment).into(),
            0b11 => Address::HL(HLMode::Decrement).into(),
            _ => unreachable!(),
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
