use crate::bus::Bus;

use crate::cartridge::CartridgeMapper;
use crate::registers::IoRegister;

pub struct Mmu {
    cartridge: Box<dyn CartridgeMapper>,
    interrupt_enable: u8, // IE
    interrupt_flag: u8,   // IF
}

impl Mmu {
    pub fn new(cartridge: Box<dyn CartridgeMapper>) -> Self {
        Self {
            cartridge,
            interrupt_enable: 0,
            interrupt_flag: 0,
        }
    }

    fn read_io(&self, address: u8) -> u8 {
        match IoRegister::try_from(address) {
            Ok(IoRegister::IF) => self.interrupt_flag,
            Ok(reg) => todo!("unhandled I/O register: {reg}"),
            Err(_) => panic!("unknown I/O register: {address:#04x}"),
        }
    }

    fn write_io(&mut self, address: u8, value: u8) {
        match IoRegister::try_from(address) {
            Ok(IoRegister::IF) => self.interrupt_flag = value,
            Ok(reg) => todo!("unhandled I/O register: {reg}"),
            Err(_) => panic!("unknown I/O register: {address:#04x}"),
        }
    }
}

impl Bus for Mmu {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7fff => self.cartridge.read_rom(address),
            0x8000..=0x9fff => todo!("{address:#06x}: read from video RAM"),
            0xa000..=0xbfff => self.cartridge.read_ram(address - 0xa000),
            0xc000..=0xcfff => todo!("{address:#06x}: read from work RAM (bank 0)"),
            0xd000..=0xdfff => todo!("{address:#06x}: read from work RAM switchable bank"),
            0xe000..=0xfdff => self.read_byte(address - 0x2000), // echo
            0xfe00..=0xfe9f => todo!("{address:#06x}: read from OAM"),
            // https://gbdev.io/pandocs/Memory_Map.html#fea0feff-range
            0xfea0..=0xfeff => todo!("{address:#06x}: read from unusable memory range"),
            0xff00..=0xff7f => self.read_io((address - 0xff00) as u8),
            0xff80..=0xfffe => todo!("{address:#06x}: read from high RAM"),
            0xffff => self.interrupt_enable,
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7fff => self.cartridge.write_rom(address, value),
            0x8000..=0x9fff => todo!("{address:#06x}: write to video RAM"),
            0xa000..=0xbfff => self.cartridge.write_ram(address - 0xa000, value),
            0xc000..=0xcfff => todo!("{address:#06x}: write to work RAM (bank 0)"),
            0xd000..=0xdfff => todo!("{address:#06x}: write to work RAM switchable bank"),
            0xe000..=0xfdff => self.write_byte(address - 0x2000, value), // echo
            0xfe00..=0xfe9f => todo!("{address:#06x}: write to OAM"),
            // https://gbdev.io/pandocs/Memory_Map.html#fea0feff-range
            0xfea0..=0xfeff => todo!("{address:#06x}: write to unusable memory range"),
            0xff00..=0xff7f => self.write_io((address - 0xff00) as u8, value),
            0xff80..=0xfffe => todo!("{address:#06x}: write to high RAM"),
            0xffff => self.interrupt_enable = value,
        }
    }
}
