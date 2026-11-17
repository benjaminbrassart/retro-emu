use super::cpu::bus::Bus;
use super::mbc::CartridgeMapper;

macro_rules! io_registers {
    ($($name:ident = $offset:literal),* $(,)?) => {
        pub enum IoRegister {
            $($name,)*
            WaveRAM(u8),
        }

        impl IoRegister {
            // pub fn as_address(&self) -> u16 {
            //     match self {
            //         $(Self::$name => 0xff00 + $offset,)*
            //         Self::WaveRAM(index) => 0xff30 + *index as u16,
            //     }
            // }
        }

        impl TryFrom<u8> for IoRegister {
            type Error = ();

            fn try_from(offset: u8) -> Result<Self, Self::Error> {
                match offset {
                    $($offset => Ok(Self::$name),)*
                    0x30..=0x3f => Ok(Self::WaveRAM(offset - 0x30)),
                    _ => Err(()),
                }
            }
        }

        impl std::fmt::Display for IoRegister {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                match self {
                    $(Self::$name => f.write_str(stringify!($name)),)*
                    Self::WaveRAM(index) => write!(f, "waveRAM[{index}]"),
                }
            }
        }
    }
}

io_registers! {
    Joyp = 0x00, Sb = 0x01, Sc = 0x02,
    Div = 0x04, Tima = 0x05, Tma = 0x06, Tac = 0x07, If = 0x0f,

    Nr10 = 0x10, Nr11 = 0x11, Nr12 = 0x12, Nr13 = 0x13, Nr14 = 0x14,
    Nr21 = 0x16, Nr22 = 0x17, Nr23 = 0x18, Nr24 = 0x19,
    Nr30 = 0x1a, Nr31 = 0x1b, Nr32 = 0x1c, Nr33 = 0x1d, Nr34 = 0x1e,
    Nr41 = 0x20, Nr42 = 0x21, Nr43 = 0x22, Nr44 = 0x23,
    Nr50 = 0x24, Nr51 = 0x25, Nr52 = 0x26,

    Lcdc = 0x40, Stat = 0x41, Scy = 0x42, Scx = 0x43,
    Ly = 0x44, Lyc = 0x45, Dma = 0x46,
    Bgp = 0x47, Obp0 = 0x48, Obp1 = 0x49,
    Wy = 0x4a, Wx = 0x4b,

    Key0 = 0x4c, Key1 = 0x4d, Vbk = 0x4f, Bank = 0x50,
    Hdma1 = 0x51, Hdma2 = 0x52, Hdma3 = 0x53,
    Hdma4 = 0x54, Hdma5 = 0x55, Rp = 0x56,
    Bcps = 0x68, Bcpd = 0x69, Ocps = 0x6a, Ocpd = 0x6b,
    Opri = 0x6c, Svbk = 0x70, Pcm12 = 0x76, Pcm34 = 0x77,
}

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

    fn tick_tcycle(&mut self) {
        self.cartridge.tick();
    }

    fn read_io(&self, address: u8) -> u8 {
        match IoRegister::try_from(address) {
            Ok(IoRegister::If) => self.interrupt_flag,
            Ok(reg) => todo!("unhandled I/O register: {reg}"),
            Err(_) => panic!("unknown I/O register: {address:#04x}"),
        }
    }

    fn write_io(&mut self, address: u8, value: u8) {
        match IoRegister::try_from(address) {
            Ok(IoRegister::If) => self.interrupt_flag = value,
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

    fn tick_mcycle(&mut self) {
        for _ in 0..4 {
            self.tick_tcycle();
        }
    }
}
