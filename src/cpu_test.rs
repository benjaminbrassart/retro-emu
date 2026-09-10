use crate::bus::Bus;
use crate::cpu::*;

struct TestBus(Vec<u8>);

impl Bus for TestBus {
    fn read_byte(&self, address: u16) -> u8 {
        self.0[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.0[address as usize] = value
    }
}

fn assert_decode(input: Vec<u8>, want: Instruction, pc: u16) {
    let mut cpu = Cpu::default();
    let bus = TestBus(input);

    let have = cpu.fetch_next_instruction(&bus);

    assert_eq!(have, want);
    assert_eq!(cpu.pc, pc, "pc register");
}

#[test]
fn decode_nop() {
    assert_decode(vec![0x00], Instruction::Nop, 1);
}

#[test]
fn decode_illegal() {
    // https://gbdev.io/pandocs/CPU_Instruction_Set.html#block-3

    let opcodes = [
        0xd3, 0xdb, 0xdd, 0xe3, 0xe4, 0xeb, 0xec, 0xed, 0xf4, 0xfc, 0xfd,
    ];

    for opcode in opcodes {
        assert_decode(vec![opcode], Instruction::Illegal { opcode }, 1);
    }
}

#[test]
fn decode_halt() {
    assert_decode(vec![0x76], Instruction::Halt, 1);
}

#[test]
fn decode_stop() {
    assert_decode(vec![0x10, 0x42], Instruction::Stop { code: 0x42 }, 2);
}

#[test]
fn decode_ld_r16_n16() {
    let inputs = [
        (0x01, Reg16::BC),
        (0x11, Reg16::DE),
        (0x21, Reg16::HL),
        (0x31, Reg16::SP),
    ];

    for (opcode, reg) in inputs {
        assert_decode(
            vec![opcode, 0xab, 0xcd],
            Instruction::Load16 {
                dst: reg.into(),
                src: Operand16::Immediate(0xcdab),
            },
            3,
        );
    }
}

#[test]
fn decode_rot_a() {
    assert_decode(vec![0x07], Instruction::Rlca, 1);
    assert_decode(vec![0x0f], Instruction::Rrca, 1);
    assert_decode(vec![0x17], Instruction::Rla, 1);
    assert_decode(vec![0x1f], Instruction::Rra, 1);
}

#[test]
fn decode_ei_di() {
    assert_decode(vec![0xf3], Instruction::DisableInterrupts, 1);
    assert_decode(vec![0xfb], Instruction::EnableInterrupts, 1);
}

#[test]
fn decode_misc() {
    assert_decode(vec![0x27], Instruction::Daa, 1);
    assert_decode(vec![0x2f], Instruction::Cpl, 1);
    assert_decode(vec![0x37], Instruction::Scf, 1);
    assert_decode(vec![0x3f], Instruction::Ccf, 1);
}

#[test]
fn decode_ret() {
    assert_decode(vec![0xc9], Instruction::Ret { condition: None }, 1);
}

#[test]
fn decode_ret_cond() {
    let inputs = [
        (0xc0, JumpCondition::NZ),
        (0xc8, JumpCondition::Z),
        (0xd0, JumpCondition::NC),
        (0xd8, JumpCondition::C),
    ];

    for (opcode, condition) in inputs {
        assert_decode(
            vec![opcode],
            Instruction::Ret {
                condition: Some(condition),
            },
            1,
        );
    }
}

#[test]
fn decode_reti() {
    assert_decode(vec![0xd9], Instruction::Reti, 1);
}

#[test]
fn decode_push() {
    let inputs = [
        (0xc5, Reg16::BC),
        (0xd5, Reg16::DE),
        (0xe5, Reg16::HL),
        (0xf5, Reg16::AF),
    ];

    for (opcode, src) in inputs {
        assert_decode(vec![opcode], Instruction::Push { src }, 1);
    }
}

#[test]
fn decode_pop() {
    let inputs = [
        (0xc1, Reg16::BC),
        (0xd1, Reg16::DE),
        (0xe1, Reg16::HL),
        (0xf1, Reg16::AF),
    ];

    for (opcode, dst) in inputs {
        assert_decode(vec![opcode], Instruction::Pop { dst }, 1);
    }
}

#[test]
fn decode_rlc() {
    let regs: [Operand8; 8] = [
        Reg8::B.into(),
        Reg8::C.into(),
        Reg8::D.into(),
        Reg8::E.into(),
        Reg8::H.into(),
        Reg8::L.into(),
        Address::Register(Reg16::HL).into(),
        Reg8::A.into(),
    ];

    for (i, &reg) in regs.iter().enumerate() {
        assert_decode(vec![0xcb, i as u8], Instruction::Rlc { src: reg }, 2);
    }
}

#[test]
fn decode_rrc() {
    let regs: [Operand8; 8] = [
        Reg8::B.into(),
        Reg8::C.into(),
        Reg8::D.into(),
        Reg8::E.into(),
        Reg8::H.into(),
        Reg8::L.into(),
        Address::Register(Reg16::HL).into(),
        Reg8::A.into(),
    ];

    for (i, &reg) in regs.iter().enumerate() {
        assert_decode(vec![0xcb, i as u8 | 0x08], Instruction::Rrc { src: reg }, 2);
    }
}

#[test]
fn decode_rl() {
    let regs: [Operand8; 8] = [
        Reg8::B.into(),
        Reg8::C.into(),
        Reg8::D.into(),
        Reg8::E.into(),
        Reg8::H.into(),
        Reg8::L.into(),
        Address::Register(Reg16::HL).into(),
        Reg8::A.into(),
    ];

    for (i, &reg) in regs.iter().enumerate() {
        assert_decode(vec![0xcb, i as u8 | 0x10], Instruction::Rl { src: reg }, 2);
    }
}

#[test]
fn decode_rr() {
    let regs: [Operand8; 8] = [
        Reg8::B.into(),
        Reg8::C.into(),
        Reg8::D.into(),
        Reg8::E.into(),
        Reg8::H.into(),
        Reg8::L.into(),
        Address::Register(Reg16::HL).into(),
        Reg8::A.into(),
    ];

    for (i, &reg) in regs.iter().enumerate() {
        assert_decode(vec![0xcb, i as u8 | 0x18], Instruction::Rr { src: reg }, 2);
    }
}

#[test]
fn decode_sla() {
    let regs: [Operand8; 8] = [
        Reg8::B.into(),
        Reg8::C.into(),
        Reg8::D.into(),
        Reg8::E.into(),
        Reg8::H.into(),
        Reg8::L.into(),
        Address::Register(Reg16::HL).into(),
        Reg8::A.into(),
    ];

    for (i, &reg) in regs.iter().enumerate() {
        assert_decode(vec![0xcb, i as u8 | 0x20], Instruction::Sla { src: reg }, 2);
    }
}

#[test]
fn decode_sra() {
    let regs: [Operand8; _] = [
        Reg8::B.into(),
        Reg8::C.into(),
        Reg8::D.into(),
        Reg8::E.into(),
        Reg8::H.into(),
        Reg8::L.into(),
        Address::Register(Reg16::HL).into(),
        Reg8::A.into(),
    ];

    for (i, &reg) in regs.iter().enumerate() {
        assert_decode(vec![0xcb, i as u8 | 0x28], Instruction::Sra { src: reg }, 2);
    }
}

#[test]
fn decode_swap() {
    let regs: [Operand8; _] = [
        Reg8::B.into(),
        Reg8::C.into(),
        Reg8::D.into(),
        Reg8::E.into(),
        Reg8::H.into(),
        Reg8::L.into(),
        Address::Register(Reg16::HL).into(),
        Reg8::A.into(),
    ];

    for (i, &reg) in regs.iter().enumerate() {
        assert_decode(
            vec![0xcb, i as u8 | 0x30],
            Instruction::Swap { src: reg },
            2,
        );
    }
}

#[test]
fn decode_srl() {
    let regs: [Operand8; _] = [
        Reg8::B.into(),
        Reg8::C.into(),
        Reg8::D.into(),
        Reg8::E.into(),
        Reg8::H.into(),
        Reg8::L.into(),
        Address::Register(Reg16::HL).into(),
        Reg8::A.into(),
    ];

    for (i, &reg) in regs.iter().enumerate() {
        assert_decode(vec![0xcb, i as u8 | 0x38], Instruction::Srl { src: reg }, 2);
    }
}

#[test]
fn decode_bit() {
    let regs: [Operand8; _] = [
        Reg8::B.into(),
        Reg8::C.into(),
        Reg8::D.into(),
        Reg8::E.into(),
        Reg8::H.into(),
        Reg8::L.into(),
        Address::Register(Reg16::HL).into(),
        Reg8::A.into(),
    ];

    for (i, &reg) in regs.iter().enumerate() {
        for bit in 0..8 {
            assert_decode(
                vec![0xcb, i as u8 | 0x40 | bit << 3],
                Instruction::Bit { bit, src: reg },
                2,
            );
        }
    }
}

#[test]
fn decode_res() {
    let regs: [Operand8; _] = [
        Reg8::B.into(),
        Reg8::C.into(),
        Reg8::D.into(),
        Reg8::E.into(),
        Reg8::H.into(),
        Reg8::L.into(),
        Address::Register(Reg16::HL).into(),
        Reg8::A.into(),
    ];

    for (i, &reg) in regs.iter().enumerate() {
        for bit in 0..8 {
            assert_decode(
                vec![0xcb, i as u8 | 0x80 | bit << 3],
                Instruction::Res { bit, src: reg },
                2,
            );
        }
    }
}

#[test]
fn decode_set() {
    let regs: [Operand8; _] = [
        Reg8::B.into(),
        Reg8::C.into(),
        Reg8::D.into(),
        Reg8::E.into(),
        Reg8::H.into(),
        Reg8::L.into(),
        Address::Register(Reg16::HL).into(),
        Reg8::A.into(),
    ];

    for (i, &reg) in regs.iter().enumerate() {
        for bit in 0..8 {
            assert_decode(
                vec![0xcb, i as u8 | 0xc0 | bit << 3],
                Instruction::Set { bit, src: reg },
                2,
            );
        }
    }
}
