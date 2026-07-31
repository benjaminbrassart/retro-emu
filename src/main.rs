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

pub struct Cartridge {
    pub rom: Box<[u8]>,
    pub ram: Box<[u8]>,
}

impl Cartridge {
    pub const ROM_BANK_SIZE: usize = 16 * 1024; // 16 KiB
    pub const RAM_BANK_SIZE: usize = 8 * 1024;  //  8 KiB

    pub fn load<R: std::io::Read>(mut r: R) -> Result<Self, LoadCartridgeError> {
        let mut rom = Vec::new();

        _ = r.read_to_end(&mut rom)?;

        if rom.len() < 336 {
            return Err(LoadCartridgeError::TruncatedHeader);
        }

        match rom[0x147] {
            0x00 => (),
            // XXX handle other controllers
            id => return Err(LoadCartridgeError::InvalidCartridgeType { id })
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

        let rom_size = Self::ROM_BANK_SIZE * rom_banks;
        let ram_size = Self::RAM_BANK_SIZE * ram_banks;

        if rom.len() != rom_size {
            return Err(LoadCartridgeError::RomSizeMismatch {
                expected: rom_size,
                actual: rom.len(),
            })
        }

        let ram = vec![0x00; ram_size];

        Ok(Self {
            rom: rom.into_boxed_slice(),
            ram: ram.into_boxed_slice(),
        })
    }

    pub fn load_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, LoadCartridgeError> {
        let f = std::fs::File::open(path)?;

        Self::load(f)
    }
}

fn main() {
    let args = std::env::args().skip(1).collect::<Box<[_]>>();

    let rom_path = if args.len() == 1 {
        args[0].to_owned()
    } else {
        panic!("usage")
    };

    drop(args);

    let cartridge = Cartridge::load_path(rom_path).unwrap();

    _ = cartridge;
}
