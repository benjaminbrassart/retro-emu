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
