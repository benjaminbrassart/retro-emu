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
