pub trait Bus {
    fn read_byte(&self, address: u16) -> u8;

    fn write_byte(&mut self, address: u16, value: u8);
}

pub trait BusExt {
    fn read_word(&self, address: u16) -> u16;

    fn write_word(&mut self, address: u16, value: u16);
}

impl<T> BusExt for T
where
    T: Bus,
{
    fn read_word(&self, address: u16) -> u16 {
        u16::from_le_bytes([
            self.read_byte(address),
            self.read_byte(address.wrapping_add(1)),
        ])
    }

    fn write_word(&mut self, address: u16, value: u16) {
        let buf = value.to_le_bytes();

        self.write_byte(address, buf[0]);
        self.write_byte(address.wrapping_add(1), buf[0]);
    }
}
