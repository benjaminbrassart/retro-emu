use crate::bus::Bus;

pub struct Mmu {}

impl Bus for Mmu {
    fn read_byte(&self, address: u16) -> u8 {
        todo!()
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        todo!()
    }
}
