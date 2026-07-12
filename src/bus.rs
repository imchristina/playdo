use crate::bus::BusTarget::Uart;
use crate::ram::*;
use crate::rom::*;

pub const ROM_ADDR: u32 = 0x1FC00000;
pub const EXPANSION_ADDR: u32 = 0x1F000000;
pub const EXPANSION_SIZE: u32 = 8192 * 1024; // 8192Kb

// Bus-slave devices are owned by the bus struct
pub struct Bus {
    ram: Ram,
    rom: Rom,
}

impl Bus {
    pub fn new(rom_path: &str) -> Self {
        Self {
            ram: Ram::new(),
            rom: Rom::from_file(rom_path).expect("Rom file not found!"),
        }
    }

    pub fn read_u32(&self, addr: u32) -> u32 {
        match self.decode_address(addr) {
            BusTarget::Ram(raw) => self.ram.read_u32(raw),
            BusTarget::Rom(offset) => self.rom.read_u32(offset),
            BusTarget::Uart(raw) => self.uart_read_stub(raw),
            BusTarget::IoStub(raw) => self.io_read_stub(raw),
            BusTarget::CacheCtrl => self.cache_ctrl_read_stub(),
            BusTarget::Expansion => 0,
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),

        }
    }

    pub fn read_u16(&self, addr: u32) -> u16 {
        match self.decode_address(addr) {
            BusTarget::Ram(raw) => self.ram.read_u16(raw),
            BusTarget::Rom(offset) => self.rom.read_u16(offset),
            BusTarget::Uart(raw) => self.uart_read_stub(raw) as u16,
            BusTarget::IoStub(raw) => self.io_read_stub(raw) as u16,
            BusTarget::CacheCtrl => self.cache_ctrl_read_stub() as u16,
            BusTarget::Expansion => 0,
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),
        }
    }

    pub fn read_u8(&self, addr: u32) -> u8 {
        match self.decode_address(addr) {
            BusTarget::Ram(raw) => self.ram.read_u8(raw),
            BusTarget::Rom(offset) => self.rom.read_u8(offset),
            BusTarget::Uart(raw) => self.uart_read_stub(raw) as u8,
            BusTarget::IoStub(raw) => self.io_read_stub(raw) as u8,
            BusTarget::CacheCtrl => self.cache_ctrl_read_stub() as u8,
            BusTarget::Expansion => 0,
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),
        }
    }

    pub fn write_u32(&mut self, addr: u32, data: u32) {
        match self.decode_address(addr) {
            BusTarget::Ram(raw) => self.ram.write_u32(raw, data),
            BusTarget::Rom(offset) => panic!("Invalid memory access! 0x{:X}", offset),
            BusTarget::Uart(raw) => self.uart_write_stub(raw, data),
            BusTarget::IoStub(raw) => self.io_write_stub(raw, data),
            BusTarget::CacheCtrl => self.cache_ctrl_write_stub(data),
            BusTarget::Expansion => {},
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),
        }
    }

    pub fn write_u16(&mut self, addr: u32, data: u16) {
        match self.decode_address(addr) {
            BusTarget::Ram(raw) => self.ram.write_u16(raw, data),
            BusTarget::Rom(offset) => panic!("Invalid memory access! 0x{:X}", offset),
            BusTarget::Uart(raw) => self.uart_write_stub(raw, data as u32),
            BusTarget::IoStub(raw) => self.io_write_stub(raw, data as u32),
            BusTarget::CacheCtrl => self.cache_ctrl_write_stub(data as u32),
            BusTarget::Expansion => {},
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),
        }
    }

    pub fn write_u8(&mut self, addr: u32, data: u8) {
        match self.decode_address(addr) {
            BusTarget::Ram(raw) => self.ram.write_u8(raw, data),
            BusTarget::Rom(offset) => panic!("Invalid memory access! 0x{:X}", offset),
            BusTarget::Uart(raw) => self.uart_write_stub(raw, data as u32),
            BusTarget::IoStub(raw) => self.io_write_stub(raw, data as u32),
            BusTarget::CacheCtrl => self.cache_ctrl_write_stub(data as u32),
            BusTarget::Expansion => {},
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),
        }
    }

    fn uart_read_stub(&self, addr: u32) -> u32 {
        println!("Unhandled UART read: {:#X}", addr);
        0
    }

    fn uart_write_stub(&self, addr: u32, data: u32) {
        println!("Unhandled UART write: {:#X}, {:#X}", addr, data);
    }

    fn io_read_stub(&self, addr: u32) -> u32 {
        println!("Unhandled MMIO read: {:#X}", addr);
        0
    }

    fn io_write_stub(&self, addr: u32, data: u32) {
        println!("Unhandled MMIO write: {:#X}, {:#X}", addr, data);
    }

    fn cache_ctrl_read_stub(&self) -> u32 {
        println!("Unhandled cache control read");
        0
    }

    fn cache_ctrl_write_stub(&self, data: u32) {
        println!("Unhandled cache control write: {:#X}", data);
    }

    fn decode_address(&self, v_addr: u32) -> BusTarget {
        // Mask off KSEG0/1 bits
        let kseg = v_addr >> 29;
        let mut addr = v_addr;
        if (kseg == 4) || (kseg == 5) {
            addr = v_addr & 0x1FFFFFFF;
        }

        if addr < RAM_SIZE {
            BusTarget::Ram(addr)
        } else if (addr >= ROM_ADDR) && (addr < ROM_ADDR + ROM_SIZE) {
            BusTarget::Rom(addr - ROM_ADDR)
        } else if (addr >= 0x1F802020) && (addr <= 0x1F80202F) {
            BusTarget::Uart(addr)
        } else if (addr >= 0x1F801000) && (addr < 0x1F803FFF) {
            BusTarget::IoStub(addr)
        } else if addr == 0xFFFE0130 {
            BusTarget::CacheCtrl
        } else if (addr >= EXPANSION_ADDR) && (addr < EXPANSION_ADDR + EXPANSION_SIZE) {
            BusTarget::Expansion
        } else {
            BusTarget::Invalid(addr)
        }
    }
}

enum BusTarget {
    Ram(u32),
    Rom(u32),
    Uart(u32),
    IoStub(u32),
    CacheCtrl,
    Expansion,
    Invalid(u32),
}
