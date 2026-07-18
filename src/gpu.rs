pub struct Gpu {

}

impl Gpu {
    pub const GP0_ADDR: u32 = 0x1F801810;
    pub const GP1_ADDR: u32 = 0x1F801814;

    pub fn new() -> Self {
        Self {

        }
    }

    fn gpuread(&self) -> u32 {
        0
    }

    fn gpustat(&self) -> u32 {
        0xFFFFFFFF
    }

    fn gp0(&mut self, data: u32) {

    }

    fn gp1(&mut self, data: u32) {

    }

    pub fn read_u32(&self, address: u32) -> u32 {
        match address {
            Self::GP0_ADDR => self.gpuread(),
            Self::GP1_ADDR => self.gpustat(),
            _ => panic!("Invalid GPU address! {:#X}", address),
        }
    }

    pub fn write_u32(&mut self, address: u32, data: u32) {
        match address {
            Self::GP0_ADDR => self.gp0(data),
            Self::GP1_ADDR => self.gp1(data),
            _ => panic!("Invalid GPU address! {:#X}", address),
        }
    }
}
