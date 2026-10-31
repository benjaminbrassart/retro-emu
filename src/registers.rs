macro_rules! io_registers {
    ($($name:ident = $offset:literal),* $(,)?) => {
        pub enum IoRegister {
            $($name,)*
            WaveRAM(u8),
        }

        impl IoRegister {
            pub fn as_address(self) -> u16 {
                match self {
                    $(Self::$name => 0xff00 + $offset,)*
                    Self::WaveRAM(index) => 0xff30 + index as u16,
                }
            }
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
    JOYP = 0x00, SB = 0x01, SC = 0x02,
    DIV = 0x04, TIMA = 0x05, TMA = 0x06, TAC = 0x07, IF = 0x0f,

    NR10 = 0x10, NR11 = 0x11, NR12 = 0x12, NR13 = 0x13, NR14 = 0x14,
    NR21 = 0x16, NR22 = 0x17, NR23 = 0x18, NR24 = 0x19,
    NR30 = 0x1a, NR31 = 0x1b, NR32 = 0x1c, NR33 = 0x1d, NR34 = 0x1e,
    NR41 = 0x20, NR42 = 0x21, NR43 = 0x22, NR44 = 0x23,
    NR50 = 0x24, NR51 = 0x25, NR52 = 0x26,

    LCDC = 0x40, STAT = 0x41, SCY = 0x42, SCX = 0x43,
    LY = 0x44, LYC = 0x45, DMA = 0x46,
    BGP = 0x47, OBP0 = 0x48, OBP1 = 0x49,
    WY = 0x4a, WX = 0x4b,

    KEY0 = 0x4c, KEY1 = 0x4d, VBK = 0x4f, BANK = 0x50,
    HDMA1 = 0x51, HDMA2 = 0x52, HDMA3 = 0x53,
    HDMA4 = 0x54, HDMA5 = 0x55, RP = 0x56,
    BCPS = 0x68, BCPD = 0x69, OCPS = 0x6a, OCPD = 0x6b,
    OPRI = 0x6c, SVBK = 0x70, PCM12 = 0x76, PCM34 = 0x77,
}

pub struct CpuRegisters {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub pc: u16,
    pub sp: u16,
}

pub struct SchedulerRegisters {
    pub spd: u8,
    pub sys: u8,
    pub wbk: u8,
    pub ie: u8,
    pub ifl: u8,
}

pub struct JoypadRegisters {
    pub joyp: u8,
}

pub struct ApuRegisters {
    pub nr10: u8,
    pub nr11: u8,
    pub nr12: u8,
    pub nr13: u8,
    pub nr14: u8,
    pub nr20: u8,
    pub nr21: u8,
    pub nr22: u8,
    pub nr23: u8,
    pub nr24: u8,
    pub nr30: u8,
    pub nr31: u8,
    pub nr32: u8,
    pub nr33: u8,
    pub nr34: u8,
    pub nr40: u8,
    pub nr41: u8,
    pub nr42: u8,
    pub nr43: u8,
    pub nr44: u8,
    pub nr50: u8,
    pub nr51: u8,
    pub nr52: u8,
    pub pcm12: u8,
    pub pcm34: u8,
}

pub struct OamRegisters {
    pub opri: u8,
}

pub struct TimerRegisters {
    pub div: u8,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
}

pub struct LcdRegisters {
    pub lcdc: u8,
    pub stat: u8,
    pub scy: u8,
    pub scx: u8,
    pub ly: u8,
    pub lyc: u8,
    pub bgp: u8,
    pub obp0: u8,
    pub obp1: u8,
    pub bgpi: u8,
    pub bgpd: u8,
    pub ogpi: u8,
    pub ogpd: u8,
}

pub struct PpuRegisters {
    pub hdma1: u8,
    pub hdma2: u8,
    pub hdma3: u8,
    pub hdma4: u8,
    pub hdma5: u8,
    pub vbk: u8,
}

// XXX rename cartridge registers according to hardware.inc
// See https://github.com/gbdev/hardware.inc

pub struct Mbc1Registers {
    pub ram_enable: u8,
    pub rom_bank_number: u8,
    pub ram_bank_number_or_rom_bank_upper_bits: u8,
    pub bank_mode_select: u8,
}

pub struct Mbc2Registers {
    pub ram_enable_rom_bank_number: u8,
}

pub struct Mbc3Registers {
    pub rtc: u8,
    pub ram_timer_enable: u8,
    pub rom_bank_number: u8,
    pub ram_bank_number_or_rtc_register_select: u8,
    pub latch_clock_data: u8,
}

pub struct Mbc5Registers {
    pub ram_enable: u8,
    pub rom_number_lsb: u8,
    pub rom_number_msb: u8,
    pub ram_bank_number: u8,
}
