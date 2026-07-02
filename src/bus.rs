use crate::rom::*;

pub const ROM_ADDR: u32 = 0xBFC00000;

// Bus-slave devices are owned by the bus struct
pub struct Bus {
    rom: Rom
}

impl Bus {
    pub fn new(rom_path: &str) -> Self {
        Self {
            rom: Rom::from_file(rom_path).expect("Rom file not found!"),
        }
    }

    pub fn read_u32(&self, addr: u32) -> u32 {
        match self.decode_address(addr) {
            BusTarget::Rom(offset) => self.rom.read_u32(offset),
            BusTarget::Io(raw) => self.read_io_u32(raw),
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),

        }
    }

    pub fn read_u16(&self, addr: u32) -> u16 {
        match self.decode_address(addr) {
            BusTarget::Rom(offset) => self.rom.read_u16(offset),
            BusTarget::Io(raw) => self.read_io_u16(raw),
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),
        }
    }

    pub fn write_u32(&self, addr: u32, data: u32) {
        match self.decode_address(addr) {
            BusTarget::Rom(offset) => panic!("Invalid memory access! 0x{:X}", offset),
            BusTarget::Io(raw) => self.write_io_u32(raw, data),
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),
        }
    }

    pub fn write_u16(&self, addr: u32, data: u16) {
        match self.decode_address(addr) {
            BusTarget::Rom(offset) => panic!("Invalid memory access! 0x{:X}", offset),
            BusTarget::Io(raw) => self.write_io_u16(raw, data),
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),
        }
    }

    pub fn read_io_u32(&self, addr: u32) -> u32 {
        match addr {
            _ => {
                println!("Unhandled MMIO read: {:#X}", addr);
                0
            }
        }
    }

    pub fn read_io_u16(&self, addr: u32) -> u16 {
        match addr {
            _ => {
                println!("Unhandled MMIO read: {:#X}", addr);
                0
            }
        }
    }

    pub fn write_io_u32(&self, addr: u32, data: u32) {
        match addr {
            _ => println!("Unhandled MMIO write: {:#X}", addr),
        }
    }

    pub fn write_io_u16(&self, addr: u32, data: u16) {
        match addr {
            _ => println!("Unhandled MMIO write: {:#X}", addr),
        }
    }

    fn decode_address(&self, addr: u32) -> BusTarget {
        if (addr >= ROM_ADDR) && (addr <= ROM_ADDR + ROM_SIZE) {
            BusTarget::Rom(addr - ROM_ADDR)
        } else if (addr >= 0x1F801000) && (addr <= 0x1FC00000) {
            BusTarget::Io(addr)
        } else {
            BusTarget::Invalid(addr)
        }
    }
}

enum BusTarget {
    Rom(u32),
    Io(u32),
    Invalid(u32),
}
