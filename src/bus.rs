use crate::rom::*;

const ROM_ADDR: u32 = 0xBFC00000;

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
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),

        }
    }

    pub fn read_u16(&self, addr: u32) -> u16 {
        match self.decode_address(addr) {
            BusTarget::Rom(offset) => self.rom.read_u16(offset),
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),
        }
    }

    pub fn write_u32(&self, addr: u32, data: u32) {
        match self.decode_address(addr) {
            BusTarget::Rom(offset) => panic!("Invalid memory access! 0x{:X}", offset),
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),
        }
    }

    pub fn write_u16(&self, addr: u32, data: u16) {
        match self.decode_address(addr) {
            BusTarget::Rom(offset) => panic!("Invalid memory access! 0x{:X}", offset),
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),
        }
    }

    fn decode_address(&self, addr: u32) -> BusTarget {
        if (addr >= ROM_ADDR) && (addr <= ROM_ADDR + ROM_SIZE) {
            BusTarget::Rom(addr - ROM_ADDR)
        } else {
            BusTarget::Invalid(addr)
        }
    }
}

enum BusTarget {
    Rom(u32),
    Invalid(u32),
}
