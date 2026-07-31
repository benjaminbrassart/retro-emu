use std::io;

pub struct Cartridge {
    pub rom: Box<[u8]>,
    pub ram: Box<[u8]>,
}

impl Cartridge {
    pub fn load<R: io::Read>(_r: R) -> io::Result<Self> {
        todo!("load cartridge from reader")
    }
}

fn main() {
    println!("Hello, world!");
}
