pub const ROM_SIZE: u32 = 512 * 1024;

pub struct Rom {
    data: [u8; ROM_SIZE as usize],
}

impl Rom {
    pub fn from_file(path: &str) -> std::io::Result<Self> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)?;
        let mut data = [0u8; ROM_SIZE as usize];

        file.read_exact(&mut data)?;

        Ok(Self { data })
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
}
