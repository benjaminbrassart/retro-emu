use crate::cpu::{JumpCondition, Operand16, Operand8, Reg16, Reg8};

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

    pub fn branch_cycles(&self) -> u8 {
        match self {
            Self::JumpRelative { .. } => 1,
            Self::JumpAbsolute { .. } => 1,
            Self::Call { .. } => 3,
            Self::Ret { condition: None } => 0,
            Self::Ret { condition: Some(_) } => 3,
            Self::JumpHL => 0,
            Self::Nop => 0,
            Self::Illegal { .. } => 0,
            Self::Stop { .. } => 0,
            Self::Halt => 0,
            Self::Load8 { .. } => 0,
            Self::Reti => 0,
            Self::Push { .. } => 0,
            Self::Pop { .. } => 0,
            Self::EnableInterrupts => 0,
            Self::DisableInterrupts => 0,
            Self::Rst { .. } => 0,
            Self::Rlca => 0,
            Self::Rrca => 0,
            Self::Rla => 0,
            Self::Rra => 0,
            Self::Daa => 0,
            Self::Cpl => 0,
            Self::Scf => 0,
            Self::Ccf => 0,
            Self::Add16 { .. } => 0,
            Self::AddSP { .. } => 0,
            Self::Load16 { .. } => 0,
            Self::LoadSPOffset { .. } => 0,
            Self::Inc8 { .. } => 0,
            Self::Dec8 { .. } => 0,
            Self::Inc16 { .. } => 0,
            Self::Dec16 { .. } => 0,
            Self::Rlc { .. } => 0,
            Self::Rrc { .. } => 0,
            Self::Rl { .. } => 0,
            Self::Rr { .. } => 0,
            Self::Sra { .. } => 0,
            Self::Sla { .. } => 0,
            Self::Swap { .. } => 0,
            Self::Srl { .. } => 0,
            Self::Bit { .. } => 0,
            Self::Res { .. } => 0,
            Self::Set { .. } => 0,
            Self::Add { .. } => 0,
            Self::Adc { .. } => 0,
            Self::Sub { .. } => 0,
            Self::Sbc { .. } => 0,
            Self::And { .. } => 0,
            Self::Xor { .. } => 0,
            Self::Or { .. } => 0,
            Self::Cp { .. } => 0,
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
