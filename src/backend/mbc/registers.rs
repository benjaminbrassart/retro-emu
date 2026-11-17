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
