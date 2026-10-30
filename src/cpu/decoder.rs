use crate::bus::Bus;
use crate::cpu::*;

pub struct Decoder {
    pc: u16,
    rc: u16,
}

impl Decoder {
    pub fn new(pc: u16) -> Self {
        Self { pc, rc: 0 }
    }

    fn get_current_pc(&self) -> u16 {
        self.pc.wrapping_add(self.rc)
    }

    fn fetch_byte<B>(&self, bus: &mut B, address: u16) -> u8
    where
        B: Bus,
    {
        let b = bus.read_byte(address);

        bus.tick_mcycle();

        b
    }

    fn fetch_next_byte<B>(&mut self, bus: &mut B) -> u8
    where
        B: Bus,
    {
        let b = self.fetch_byte(bus, self.get_current_pc());

        self.rc = self.rc.wrapping_add(1);

        b
    }

    fn fetch_next_word<B>(&mut self, bus: &mut B) -> u16
    where
        B: Bus,
    {
        u16::from_le_bytes([self.fetch_next_byte(bus), self.fetch_next_byte(bus)])
    }

    pub fn decode_instruction<B>(&mut self, bus: &mut B) -> (Instruction, u16)
    where
        B: Bus,
    {
        let opcode = self.fetch_next_byte(bus);

        let instruction = match opcode {
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

            0xcb => self.decode_prefixed_instruction(bus),
        };

        (instruction, self.rc)
    }

    pub fn decode_prefixed_instruction<B>(&mut self, bus: &mut B) -> Instruction
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
}

#[cfg(test)]
mod tests {
    struct TestBus {
        pub ticks: u8,
        pub memory: Box<[u8]>,
    }

    impl TestBus {
        pub fn unprefixed(opcode: u8) -> Self {
            Self {
                ticks: 0,
                memory: vec![opcode].into_boxed_slice(),
            }
        }

        pub fn prefixed(opcode: u8) -> Self {
            Self {
                ticks: 0,
                memory: vec![0xcb, opcode].into_boxed_slice(),
            }
        }
    }

    impl crate::bus::Bus for TestBus {
        fn read_byte(&self, address: u16) -> u8 {
            if let Some(b) = self.memory.get(address as usize) {
                *b
            } else {
                address as u8
            }
        }

        fn write_byte(&mut self, _address: u16, _value: u8) {}

        fn tick_mcycle(&mut self) {
            self.ticks += 1;
        }
    }

    // Generated from https://gbdev.io/gb-opcodes/Opcodes.json
    const UNPREFIXED_MCYCLES: [u8; 256] = [
        1, 3, 1, 1, 1, 1, 2, 1, 3, 1, 1, 1, 1, 1, 2, 1, 2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1,
        2, 1, 2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, 2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1,
        1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 1, 2, 1, 1, 1, 3, 1, 3, 3, 2, 1, 1, 1,
        3, 1, 3, 1, 2, 1, 1, 1, 3, 1, 3, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, 2, 1, 3, 1, 1, 1, 2, 1,
        2, 1, 1, 1, 1, 1, 2, 1, 2, 1, 3, 1, 1, 1, 2, 1,
    ];

    // Generated from https://gbdev.io/gb-opcodes/Opcodes.json
    const PREFIXED_MCYCLES: [u8; 256] = [
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    ];

    #[test]
    fn decode_cycles_unprefixed() {
        for opcode in 0..=255 {
            if opcode == 0xcb {
                continue;
            }

            let mut decoder = super::Decoder::new(0);
            let mut bus = TestBus::unprefixed(opcode);

            _ = decoder.decode_instruction(&mut bus);

            assert_eq!(
                bus.ticks, UNPREFIXED_MCYCLES[opcode as usize],
                "opcode = {opcode:#04x}"
            );
        }
    }

    #[test]
    fn decode_cycles_prefixed() {
        for opcode in 0..=255 {
            let mut decoder = super::Decoder::new(0);
            let mut bus = TestBus::prefixed(opcode);

            _ = decoder.decode_instruction(&mut bus);

            assert_eq!(
                bus.ticks, PREFIXED_MCYCLES[opcode as usize],
                "opcode = {opcode:#04x}"
            );
        }
    }
}
