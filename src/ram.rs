pub const RAM_SIZE: u32 = 2 * 1024 * 1024;

pub struct Ram {
    data: [u8; RAM_SIZE as usize],
}

impl Ram {
    pub fn new() -> Self {
        return Self {
            data: [0; RAM_SIZE as usize],
        }
    }

    pub fn read_u32(&self, offset: u32) -> u32 {
        let idx = offset as usize;
        let bytes: [u8; 4] = self.data[idx..idx + 4].try_into().unwrap();
        u32::from_le_bytes(bytes)
    }

    pub fn read_u16(&self, offset: u32) -> u16 {
        let idx = offset as usize;
        let bytes: [u8; 2] = self.data[idx..idx + 2].try_into().unwrap();
        u16::from_le_bytes(bytes)
    }

    pub fn write_u32(&mut self, offset: u32, data: u32) {
        let idx = offset as usize;
        let bytes = data.to_le_bytes();
        self.data[idx..idx + 4].copy_from_slice(&bytes);
    }

    pub fn write_u16(&mut self, offset: u32, data: u16) {
        let idx = offset as usize;
        let bytes = data.to_le_bytes();
        self.data[idx..idx + 2].copy_from_slice(&bytes);
    }
}
