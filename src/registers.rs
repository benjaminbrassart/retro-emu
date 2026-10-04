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
