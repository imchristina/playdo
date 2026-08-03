use crate::interrupt::Interrupt;

#[derive(Default)]
pub struct Gpu {
    pub texture_page_x_base: u32,
    pub texture_page_y_base: u32,
    pub semi_transparency: u32,
    pub texture_page_colors: u32,
    pub dither: bool,
    pub drawing: bool,
    pub set_mask: bool,
    pub mask_override: bool,
    pub field: bool,
    pub h_res: u32,
    pub v_res: bool,
    pub video_mode: bool,
    pub color_depth: bool,
    pub interlace: bool,
    pub display_disable: bool,
    pub irq1: bool,
    pub dma: u32,
    pub cmd_ready: bool,
    pub cpu_read_ready: bool,
    pub dma_ready: bool,
    pub interlace_even_odd: bool,
    pub textured_rectangle_x_flip: bool,
    pub textured_rectangle_y_flip: bool,
}

impl Gpu {
    pub const GP0_ADDR: u32 = 0x1F801810;
    pub const GP1_ADDR: u32 = 0x1F801814;

    pub fn new() -> Self {
        let mut out = Self::default();
        out.display_disable = true;
        return out
    }

    fn gpuread(&self) -> u32 {
        0
    }

    pub fn gpustat(&self) -> u32 {
        let mut gpustat = 0;

        let bit25 = match self.dma {
            0 => false,
            1 => true, // FIFO state
            2 => self.dma_ready,
            3 => self.cpu_read_ready,
            _ => panic!("Unknown DMA state! {:#X}", self.dma),
        };

        gpustat |= self.texture_page_x_base & 0b111;
        gpustat |= (self.texture_page_y_base & 0b1) << 4;
        gpustat |= (self.semi_transparency & 0b11) << 5;
        gpustat |= (self.texture_page_colors & 0b11) << 7;
        gpustat |= (self.dither as u32) << 9;
        gpustat |= (self.drawing as u32) << 10;
        gpustat |= (self.set_mask as u32) << 11;
        gpustat |= (self.mask_override as u32) << 12;
        gpustat |= (self.field as u32) << 13;
        // Bit 14 not used on retail hardware
        gpustat |= ((self.texture_page_y_base & 0b10) >> 1) << 15;
        gpustat |= (self.h_res & 0b111) << 16;
        gpustat |= (self.v_res as u32) << 19;
        gpustat |= (self.video_mode as u32) << 20;
        gpustat |= (self.color_depth as u32) << 21;
        gpustat |= (self.interlace as u32) << 22;
        gpustat |= (self.display_disable as u32) << 23;
        gpustat |= (self.irq1 as u32) << 24;
        gpustat |= (bit25 as u32) << 25;
        gpustat |= (self.cmd_ready as u32) << 26;
        gpustat |= (self.cpu_read_ready as u32) << 27;
        gpustat |= (self.dma_ready as u32) << 28;
        gpustat |= (self.dma & 0b11) << 29;
        gpustat |= (self.interlace_even_odd as u32) << 31;

        return gpustat
    }

    fn gp0(&mut self, gp0: Gp0) {
        match gp0.command() {
            7 => match gp0.environment() {
                0xE1 => self.gp0_drawmode(gp0),
                _ => panic!("Unknown GP0 environment! Environment: {:#X}, Raw: {:#X}", gp0.environment(), gp0.0)
            }
            _ => panic!("Unknown GP0 command! Command: {}, Raw: {:#X}", gp0.command(), gp0.0)
        }
    }

    fn gp0_drawmode(&mut self, gp0: Gp0) {
        self.texture_page_x_base = gp0.0 & 0b1111;
        self.texture_page_y_base = gp0.0 >> 4;
        self.semi_transparency = (gp0.0 >> 5) & 0b11;
        self.texture_page_colors = (gp0.0 >> 7) & 0b11;
        self.dither = (gp0.0 >> 9) != 0;
        self.drawing = (gp0.0 >> 10) != 0;
        // Bit 11 texture page Y base 2, unused on retail hardware
        self.textured_rectangle_x_flip = (gp0.0 >> 12) != 0;
        self.textured_rectangle_y_flip = (gp0.0 >> 13) != 0;
    }

    fn gp1(&mut self, data: u32) {

    }

    pub fn step(&mut self, interrupt: &mut Interrupt) {

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
            Self::GP0_ADDR => self.gp0(Gp0(data)),
            Self::GP1_ADDR => self.gp1(data),
            _ => panic!("Invalid GPU address! {:#X}", address),
        }
    }
}

struct Gp0(u32);

impl Gp0 {
    fn command(&self) -> u32 {
        self.0 >> 29
    }

    fn environment(&self) -> u32 {
        self.0 >> 24
    }
}
