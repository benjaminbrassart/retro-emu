#[derive(Debug)]
pub enum LoadCartridgeError {
    Io {
        err: std::io::Error,
    },

    TruncatedHeader,

    RomSizeMismatch {
        expected: usize,
        actual: usize,
    },

    InvalidCartridgeType {
        id: u8,
    },

    InvalidRomSize {
        id: u8,
    },

    InvalidRamSize {
        id: u8,
    },
}

impl std::fmt::Display for LoadCartridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Io { err } => write!(f, "{err}"),
            Self::TruncatedHeader => write!(f, "header truncated"),
            Self::RomSizeMismatch { expected, actual } => write!(f, "rom size mismatch: expected {expected}, got {actual} bytes"),
            Self::InvalidCartridgeType { id } => write!(f, "invalid cartridge type: {id:02x}"),
            Self::InvalidRomSize { id } => write!(f, "invalid rom size: {id:02x}"),
            Self::InvalidRamSize { id } => write!(f, "invalid ram size: {id:02x}"),
        }
    }
}

impl std::error::Error for LoadCartridgeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { err } => Some(err),
            Self::TruncatedHeader => None,
            Self::RomSizeMismatch { .. } => None,
            Self::InvalidCartridgeType { .. } => None,
            Self::InvalidRomSize { .. } => None,
            Self::InvalidRamSize { .. } => None,
        }
    }
}

impl From<std::io::Error> for LoadCartridgeError {
    fn from(err: std::io::Error) -> Self {
        Self::Io { err }
    }
}

pub trait CartridgeMapper {
    fn read_rom(&self, address: usize) -> u8;

    fn read_ram(&self, address: usize) -> u8;

    fn write_rom(&mut self, address: usize, value: u8);

    fn write_ram(&mut self, address: usize, value: u8);
}

pub struct PlainCartridgeMapper {
    pub rom: Box<[u8]>,
    pub ram: Box<[u8]>,
}

impl CartridgeMapper for PlainCartridgeMapper {
    fn read_rom(&self, address: usize) -> u8 {
        *self.rom.get(address).unwrap_or(&0xff)
    }

    fn read_ram(&self, address: usize) -> u8 {
        *self.ram.get(address).unwrap_or(&0xff)
    }

    fn write_rom(&mut self, _: usize, _: u8) {
    }

    fn write_ram(&mut self, address: usize, value: u8) {
        if let Some(b) = self.ram.get_mut(address) {
            *b = value
        }
    }
}

pub fn load_cartridge<R: std::io::Read>(mut r: R) -> Result<Box<dyn CartridgeMapper>, LoadCartridgeError> {
    const ROM_BANK_SIZE: usize = 16 * 1024; // 16 KiB
    const RAM_BANK_SIZE: usize = 8 * 1024;  //  8 KiB

    let mut rom = Vec::new();

    _ = r.read_to_end(&mut rom)?;

    if rom.len() < 336 {
        return Err(LoadCartridgeError::TruncatedHeader);
    }

    let rom_banks = match rom[0x148] {
        0x00 => 2,
        0x01 => 4,
        0x02 => 8,
        0x03 => 16,
        0x04 => 32,
        0x05 => 64,
        0x06 => 128,
        0x07 => 256,
        0x08 => 512,
        id => return Err(LoadCartridgeError::InvalidRomSize { id })
    };

    let ram_banks = match rom[0x149] {
        0x00 => 0,
        0x02 => 1,
        0x03 => 4,
        0x04 => 16,
        0x05 => 8,
        id => return Err(LoadCartridgeError::InvalidRamSize { id })
    };

    let rom_size = ROM_BANK_SIZE * rom_banks;
    let ram_size = RAM_BANK_SIZE * ram_banks;

    if rom.len() != rom_size {
        return Err(LoadCartridgeError::RomSizeMismatch {
            expected: rom_size,
            actual: rom.len(),
        })
    }

    let ram = vec![0x00; ram_size];

    let mapper: Box<dyn CartridgeMapper> = match rom[0x147] {
        0x00 => Box::new(PlainCartridgeMapper {
            rom: rom.into_boxed_slice(),
            ram: ram.into_boxed_slice(),
        }),

        // XXX handle other controllers

        id => return Err(LoadCartridgeError::InvalidCartridgeType { id })
    };

    Ok(mapper)
}

pub fn load_cartridge_path<P: AsRef<std::path::Path>>(path: P) -> Result<Box<dyn CartridgeMapper>, LoadCartridgeError> {
    let f = std::fs::File::open(path)?;

    load_cartridge(f)
}
