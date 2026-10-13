#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CpuFlags {
    pub zero: bool,
    pub sub: bool,
    pub half: bool,
    pub carry: bool,
}

impl CpuFlags {
    const Z: u8 = 0b1000_0000;
    const N: u8 = 0b0100_0000;
    const H: u8 = 0b0010_0000;
    const C: u8 = 0b0001_0000;
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

impl Into<u8> for CpuFlags {
    fn into(self) -> u8 {
        let zero = if self.zero { Self::Z } else { 0 };
        let sub = if self.sub { Self::N } else { 0 };
        let half = if self.half { Self::H } else { 0 };
        let carry = if self.carry { Self::C } else { 0 };

        zero | sub | half | carry
    }
}
