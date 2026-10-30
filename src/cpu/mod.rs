use crate::bus::{Bus, BusExt};

mod flags;
pub use flags::*;

mod operands;
pub use operands::*;

mod instructions;
pub use instructions::*;

mod decoder;
use decoder::Decoder;

mod arithmetic;

mod executor;

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

    pub fn fetch_next_instruction<B>(&mut self, bus: &mut B) -> Instruction
    where
        B: Bus,
    {
        let mut decoder = Decoder::new(self.pc);
        let (instruction, rc) = decoder.decode_instruction(bus);

        self.pc = self.pc.wrapping_add(rc);

        instruction
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
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
