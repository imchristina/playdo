#[derive(Default)]
pub struct Interrupt {
    pub i_stat: u32,
    pub i_mask: u32,
}

impl Interrupt {
    pub const I_STAT_ADDR: u32 = 0x1F801070;
    pub const I_MASK_ADDR: u32 = 0x1F801074;

    pub fn read_u32(&self, address: u32) -> u32 {
        match address {
            Self::I_STAT_ADDR => return self.i_stat,
            Self::I_MASK_ADDR => return self.i_mask,
            _ => panic!("Invalid interrupt address! {:#X}", address),
        }
    }

    pub fn read_u16(&self, address: u32) -> u16 {
        match address {
            Self::I_STAT_ADDR => return self.i_stat as u16,
            Self::I_MASK_ADDR => return self.i_mask as u16,
            _ => panic!("Invalid interrupt address! {:#X}", address),
        }
    }

    pub fn read_u8(&self, address: u32) -> u8 {
        match address {
            Self::I_STAT_ADDR => return self.i_stat as u8,
            Self::I_MASK_ADDR => return self.i_mask as u8,
            _ => panic!("Invalid interrupt address! {:#X}", address),
        }
    }

    pub fn write_u32(&mut self, address: u32, data: u32) {
        match address {
            Self::I_STAT_ADDR => self.i_stat = data,
            Self::I_MASK_ADDR => self.i_mask = data,
            _ => panic!("Invalid interrupt address! {:#X}", address),
        }
    }
}
