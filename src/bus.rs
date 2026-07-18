use crate::ram::*;
use crate::rom::*;
use crate::interrupt::*;
use crate::gpu::*;

pub const ROM_ADDR: u32 = 0x1FC00000;
pub const EXPANSION_ADDR: u32 = 0x1F000000;
pub const EXPANSION_SIZE: u32 = 8192 * 1024; // 8192Kb

// Bus-slave devices are owned by the bus struct
pub struct Bus {
    ram: Ram,
    rom: Rom,
    pub interrupt: Interrupt,
    gpu: Gpu,
}

impl Bus {
    pub fn new(rom_path: &str) -> Self {
        Self {
            ram: Ram::new(),
            rom: Rom::from_file(rom_path).expect("Rom file not found!"),
            interrupt: Interrupt::default(),
            gpu: Gpu::new(),
        }
    }

    pub fn read_u32(&self, addr: u32) -> u32 {
        match self.decode_address(addr) {
            BusTarget::Ram(raw) => self.ram.read_u32(raw),
            BusTarget::Rom(offset) => self.rom.read_u32(offset),
            BusTarget::Interrupt(raw) => self.interrupt.read_u32(raw),
            BusTarget::Gpu(raw) => self.gpu.read_u32(raw),
            BusTarget::Serial(raw) => self.serial_read_stub(raw),
            BusTarget::IoStub(raw) => self.io_read_stub(raw),
            BusTarget::CacheCtrl => self.cache_ctrl_read_stub(),
            BusTarget::Expansion => 0,
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),
        }
    }
    pub fn read_u32_debug(&self, addr: u32) -> u32 {
        match self.decode_address(addr) {
            BusTarget::Ram(raw) => self.ram.read_u32(raw),
            BusTarget::Rom(offset) => self.rom.read_u32(offset),
            BusTarget::Interrupt(raw) => self.interrupt.read_u32(raw),
            BusTarget::Gpu(raw) => self.gpu.read_u32(raw),
            BusTarget::Serial(raw) => self.serial_read_stub(raw),
            BusTarget::IoStub(raw) => self.io_read_stub(raw),
            BusTarget::CacheCtrl => self.cache_ctrl_read_stub(),
            BusTarget::Expansion => 0,
            BusTarget::Invalid(_raw) => 0,
        }
    }

    pub fn read_u16(&self, addr: u32) -> u16 {
        match self.decode_address(addr) {
            BusTarget::Ram(raw) => self.ram.read_u16(raw),
            BusTarget::Rom(offset) => self.rom.read_u16(offset),
            BusTarget::Interrupt(raw) => self.interrupt.read_u16(raw),
            BusTarget::Gpu(raw) => self.gpu.read_u32(raw) as u16,
            BusTarget::Serial(raw) => self.serial_read_stub(raw) as u16,
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
            BusTarget::Interrupt(raw) => self.interrupt.read_u8(raw),
            BusTarget::Gpu(raw) => self.gpu.read_u32(raw) as u8,
            BusTarget::Serial(raw) => self.serial_read_stub(raw) as u8,
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
            BusTarget::Interrupt(raw) => self.interrupt.write_u32(raw, data),
            BusTarget::Gpu(raw) => self.gpu.write_u32(raw, data),
            BusTarget::Serial(raw) => self.serial_write_stub(raw, data),
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
            BusTarget::Interrupt(raw) => self.interrupt.write_u32(raw, data as u32),
            BusTarget::Gpu(raw) => self.gpu.write_u32(raw, data as u32),
            BusTarget::Serial(raw) => self.serial_write_stub(raw, data as u32),
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
            BusTarget::Interrupt(raw) => self.interrupt.write_u32(raw, data as u32),
            BusTarget::Gpu(raw) => self.gpu.write_u32(raw, data as u32),
            BusTarget::Serial(raw) => self.serial_write_stub(raw, data as u32),
            BusTarget::IoStub(raw) => self.io_write_stub(raw, data as u32),
            BusTarget::CacheCtrl => self.cache_ctrl_write_stub(data as u32),
            BusTarget::Expansion => {},
            BusTarget::Invalid(raw) => panic!("Invalid memory access! 0x{:X}", raw),
        }
    }

    fn serial_read_stub(&self, addr: u32) -> u32 {
        //println!("Unhandled serial read: {:#X}", addr);
        0
    }

    fn serial_write_stub(&self, addr: u32, data: u32) {
        //println!("Unhandled serial write: {:#X}, {:#X}", addr, data);
    }

    fn io_read_stub(&self, addr: u32) -> u32 {
        //println!("Unhandled MMIO read: {:#X}", addr);
        0
    }

    fn io_write_stub(&self, addr: u32, data: u32) {
        //println!("Unhandled MMIO write: {:#X}, {:#X}", addr, data);
    }

    fn cache_ctrl_read_stub(&self) -> u32 {
        //println!("Unhandled cache control read");
        0
    }

    fn cache_ctrl_write_stub(&self, data: u32) {
        //println!("Unhandled cache control write: {:#X}", data);
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
        } else if (addr == Interrupt::I_STAT_ADDR) || (addr == Interrupt::I_MASK_ADDR) {
            BusTarget::Interrupt(addr)
        } else if (addr == Gpu::GP0_ADDR) || (addr == Gpu::GP1_ADDR) {
            BusTarget::Gpu(addr)
        } else if (addr >= 0x1F802020) && (addr <= 0x1F80202F) {
            BusTarget::Serial(addr)
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
    Interrupt(u32),
    Gpu(u32),
    Serial(u32),
    IoStub(u32),
    CacheCtrl,
    Expansion,
    Invalid(u32),
}
