pub trait Bus {
    fn read_byte(&self, address: u16) -> u8;

    fn write_byte(&mut self, address: u16, value: u8);

    fn tick_mcycle(&mut self);
}
