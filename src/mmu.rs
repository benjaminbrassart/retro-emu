use crate::bus::Bus;

use crate::cartridge::CartridgeMapper;

pub struct Mmu {
    cartridge: Box<dyn CartridgeMapper>,
}

impl Mmu {
    pub fn new(cartridge: Box<dyn CartridgeMapper>) -> Self {
        Self { cartridge }
    }
}

impl Bus for Mmu {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7fff => self.cartridge.read_rom(address),
            0x8000..=0x9fff => todo!("read from video RAM"),
            0xa000..=0xbfff => self.cartridge.read_ram(address - 0xa000),
            0xc000..=0xcfff => todo!("read from work RAM (bank 0)"),
            0xd000..=0xdfff => todo!("read from work RAM (dmg: bank 1, cgb: bank 1-7)"),
            0xe000..=0xfdff => self.read_byte(address - 0x2000), // echo
            0xfe00..=0xfe9f => todo!("read from OAM"),
            // https://gbdev.io/pandocs/Memory_Map.html#fea0feff-range
            0xfea0..=0xfeff => todo!("read from unusable memory range"),
            0xff00..=0xff7f => todo!("read from I/O registers"),
            0xff80..=0xfffe => todo!("read from high RAM"),
            0xffff => todo!("read IE"),
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7fff => self.cartridge.write_rom(address, value),
            0x8000..=0x9fff => todo!("write to video RAM"),
            0xa000..=0xbfff => self.cartridge.write_ram(address - 0xa000, value),
            0xc000..=0xcfff => todo!("write to work RAM (bank 0)"),
            0xd000..=0xdfff => todo!("write to work RAM (dmg: bank 1, cgb: bank 1-7)"),
            0xe000..=0xfdff => self.write_byte(address - 0x2000, value), // echo
            0xfe00..=0xfe9f => todo!("write to OAM"),
            // https://gbdev.io/pandocs/Memory_Map.html#fea0feff-range
            0xfea0..=0xfeff => todo!("write to unusable memory range"),
            0xff00..=0xff7f => todo!("write to I/O registers"),
            0xff80..=0xfffe => todo!("write to high RAM"),
            0xffff => todo!("write IE"),
        }
    }
}
