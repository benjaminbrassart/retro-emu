pub fn rl(src: u8, carry: bool) -> (u8, bool) {
    let carry_in = if carry { 0b0000_0001 } else { 0b0000_0000 };
    let res = src << 1 | carry_in;
    let carry_out = (src & 0b1000_0000) == 0b1000_0000;

    (res, carry_out)
}

pub fn rr(src: u8, carry: bool) -> (u8, bool) {
    let carry_out = if carry { 0b1000_0000 } else { 0b0000_0000 };

    (carry_out | src >> 1, src & 0b0000_0001 == 0b0000_0001)
}

pub fn rlc(src: u8) -> (u8, bool) {
    (src.rotate_left(1), src & 0b1000_0000 == 0b1000_0000)
}

pub fn rrc(src: u8) -> (u8, bool) {
    (src.rotate_right(1), src & 0b0000_0001 == 0b0000_0001)
}

pub fn sla(src: u8) -> (u8, bool) {
    let carry_out = src & 0b1000_0000 == 0b1000_0000;

    (src << 1, carry_out)
}

pub fn sra(src: u8) -> (u8, bool) {
    let carry_out = src & 0b0000_0001 == 0b0000_0001;

    (src >> 1 | src & 0b1000_0000, carry_out)
}

pub fn srl(src: u8) -> (u8, bool) {
    let carry_out = src & 0b0000_0001 == 0b0000_0001;

    (src >> 1, carry_out)
}

pub fn add(lhs: u8, rhs: u8) -> (u8, bool, bool) {
    let (res, carry) = lhs.overflowing_add(rhs);
    let half_carry = (lhs & 0x0f) + (rhs & 0x0f) > 0x0f;

    (res, half_carry, carry)
}

pub fn adc(lhs: u8, rhs: u8, carry: bool) -> (u8, bool, bool) {
    let (res, carry) = lhs.carrying_add(rhs, carry);
    let (res_half, _) = (lhs & 0x0f).carrying_add(rhs & 0x0f, carry);
    let half_carry = res_half > 0x0f;

    (res, half_carry, carry)
}

pub fn sub(lhs: u8, rhs: u8) -> (u8, bool, bool) {
    let (res, carry) = lhs.overflowing_sub(rhs);
    let half_carry = (lhs & 0x0f) < (rhs & 0x0f);

    (res, half_carry, carry)
}

pub fn sbc(lhs: u8, rhs: u8, carry: bool) -> (u8, bool, bool) {
    let carry = u8::from(carry);
    let res = lhs.wrapping_sub(rhs).wrapping_sub(carry);
    let half_carry = (lhs & 0x0f) < ((rhs & 0x0f) + carry);
    let carry_out = (lhs as u16) < (rhs as u16 + carry as u16);

    (res, half_carry, carry_out)
}

pub fn add16(lhs: u16, rhs: u16) -> (u16, bool, bool) {
    let (res, carry) = lhs.overflowing_add(rhs);
    let half_carry = (lhs & 0x0fff) + (rhs & 0x0fff) > 0x0fff;

    (res, half_carry, carry)
}

pub fn add16_signed(lhs: u16, rhs: i8) -> (u16, bool, bool) {
    let res = lhs.wrapping_add_signed(rhs as i16);
    let half_carry = (lhs & 0x000f) + u16::from((rhs as u8) & 0x0f) > 0x000f;
    let carry = (lhs & 0x00ff) + u16::from(rhs as u8) > 0x00ff;

    (res, half_carry, carry)
}
