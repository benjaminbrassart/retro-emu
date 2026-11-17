#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CpuFlags {
    pub zero: bool,
    pub sub: bool,
    pub half: bool,
    pub carry: bool,
}

impl CpuFlags {
    pub(crate) const Z: u8 = 0b1000_0000;
    pub(crate) const N: u8 = 0b0100_0000;
    pub(crate) const H: u8 = 0b0010_0000;
    pub(crate) const C: u8 = 0b0001_0000;
}

impl From<u8> for CpuFlags {
    fn from(b: u8) -> Self {
        Self {
            zero: b & Self::Z == Self::Z,
            sub: b & Self::N == Self::N,
            half: b & Self::H == Self::H,
            carry: b & Self::C == Self::C,
        }
    }
}

impl From<CpuFlags> for u8 {
    fn from(flags: CpuFlags) -> u8 {
        let zero = if flags.zero { CpuFlags::Z } else { 0 };
        let sub = if flags.sub { CpuFlags::N } else { 0 };
        let half = if flags.half { CpuFlags::H } else { 0 };
        let carry = if flags.carry { CpuFlags::C } else { 0 };

        zero | sub | half | carry
    }
}
