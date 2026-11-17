pub struct OamRegisters {
    pub opri: u8,
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
