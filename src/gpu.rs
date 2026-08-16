use crate::interrupt::Interrupt;

#[derive(Default)]
pub struct Gpu {
    pub interrupt: bool,
    pub texture_page_x_base: u32,
    pub texture_page_y_base: u32,
    pub semi_transparency: u32,
    pub texture_depth: u32,
    pub dithering: bool,
    pub draw_to_display: bool,
    pub force_set_mask_bit: bool,
    pub preserve_masked_pixels: bool,
    pub field: bool,
    pub hres: u32,
    pub vres: bool,
    pub vmode: bool,
    pub display_depth: bool,
    pub interlaced: bool,
    pub display_disable: bool,
    pub irq1: bool,
    pub dma_direction: u32,
    pub cmd_ready: bool,
    pub cpu_read_ready: bool,
    pub dma_ready: bool,
    pub interlace_even_odd: bool,
    pub rectangle_texture_x_flip: bool,
    pub rectangle_texture_y_flip: bool,
    pub texture_window_x_mask: u32,
    pub texture_window_y_mask: u32,
    pub texture_window_x_offset: u32,
    pub texture_window_y_offset: u32,
    pub drawing_area_top: u32,
    pub drawing_area_bottom: u32,
    pub drawing_area_left: u32,
    pub drawing_area_right: u32,
    pub drawing_x_offset: i16,
    pub drawing_y_offset: i16,
    pub display_vram_x_start: u32,
    pub display_vram_y_start: u32,
    pub display_h_start: u32,
    pub display_h_end: u32,
    pub display_line_start: u32,
    pub display_line_end: u32,
}

impl Gpu {
    pub const GP0_ADDR: u32 = 0x1F801810;
    pub const GP1_ADDR: u32 = 0x1F801814;

    pub fn new() -> Self {
        let mut out = Self::default();
        out.display_disable = true;

        out.cmd_ready = true;
        out.cpu_read_ready = true;
        out.dma_ready = true;
        return out
    }

    fn gpuread(&self) -> u32 {
        0
    }

    pub fn gpustat(&self) -> u32 {
        let mut gpustat = 0;

        let bit25 = match self.dma_direction {
            0 => false,
            1 => true, // FIFO state
            2 => self.dma_ready,
            3 => self.cpu_read_ready,
            _ => panic!("Unknown DMA state! {:#X}", self.dma_direction),
        };

        gpustat |= self.texture_page_x_base & 0b111;
        gpustat |= (self.texture_page_y_base & 0b1) << 4;
        gpustat |= (self.semi_transparency & 0b11) << 5;
        gpustat |= (self.texture_depth & 0b11) << 7;
        gpustat |= (self.dithering as u32) << 9;
        gpustat |= (self.draw_to_display as u32) << 10;
        gpustat |= (self.force_set_mask_bit as u32) << 11;
        gpustat |= (self.preserve_masked_pixels as u32) << 12;
        gpustat |= (self.field as u32) << 13;
        // Bit 14 not used on retail hardware
        gpustat |= ((self.texture_page_y_base & 0b10) >> 1) << 15;
        gpustat |= (self.hres & 0b111) << 16;
        gpustat |= (self.vres as u32) << 19;
        gpustat |= (self.vmode as u32) << 20;
        gpustat |= (self.display_depth as u32) << 21;
        gpustat |= (self.interlaced as u32) << 22;
        gpustat |= (self.display_disable as u32) << 23;
        gpustat |= (self.irq1 as u32) << 24;
        gpustat |= (bit25 as u32) << 25;
        gpustat |= (self.cmd_ready as u32) << 26;
        gpustat |= (self.cpu_read_ready as u32) << 27;
        gpustat |= (self.dma_ready as u32) << 28;
        gpustat |= (self.dma_direction & 0b11) << 29;
        gpustat |= (self.interlace_even_odd as u32) << 31;

        return gpustat
    }

    fn gp0(&mut self, ins: Instruction) {
        match ins.opcode() {
            0x00 => (), // NOP
            0xE1 => self.gp0_drawmode(ins),
            _ => panic!("Unknown GP0 instruction! Opcode: {}, Raw: {:#X}", ins.opcode(), ins.0)
        }
    }

    fn gp0_drawmode(&mut self, ins: Instruction) {
        self.texture_page_x_base = ins.0 & 0b1111;
        self.texture_page_y_base = (ins.0 >> 4) & 0b1;
        self.semi_transparency = (ins.0 >> 5) & 0b11;
        self.texture_depth = (ins.0 >> 7) & 0b11;
        self.dithering = ((ins.0 >> 9) & 0b1) != 0;
        self.draw_to_display = ((ins.0 >> 10) & 0b1) != 0;
        // Bit 11 texture page Y base 2, unused on retail hardware
        self.rectangle_texture_x_flip = (ins.0 >> 12) != 0;
        self.rectangle_texture_y_flip = (ins.0 >> 13) != 0;
    }

    fn gp1(&mut self, ins: Instruction) {
        match ins.opcode() {
            0x00 => self.gp1_reset(),
            0x08 => self.gp1_display_mode(ins),
            _ => panic!("Unknown GP1 instruction! Opcode: {}, Raw: {:#X}", ins.opcode(), ins.0)
        }
    }

    fn gp1_reset(&mut self) {
        // TODO clear FIFO
        self.interrupt = false;

        self.texture_page_x_base = 0;
        self.texture_page_y_base = 0;
        self.semi_transparency = 0;
        self.texture_depth = 0;
        self.texture_window_x_mask = 0;
        self.texture_window_y_mask = 0;
        self.texture_window_x_offset = 0;
        self.texture_window_y_offset = 0;
        self.dithering = false;
        self.draw_to_display = false;
        self.rectangle_texture_x_flip = false;
        self.rectangle_texture_y_flip = false;
        self.drawing_area_top = 0;
        self.drawing_area_bottom = 0;
        self.drawing_area_left = 0;
        self.drawing_area_right = 0;
        self.drawing_x_offset = 0;
        self.drawing_y_offset = 0;
        self.force_set_mask_bit = false;
        self.preserve_masked_pixels = false;
        self.dma_direction = 0;
        self.display_disable = true;
        self.display_vram_x_start = 0;
        self.display_vram_y_start = 0;
        self.hres = 0;
        self.vres = false;
        self.vmode = false;
        self.interlaced = true;
        self.display_h_start = 0x200;
        self.display_h_end = 0xc00;
        self.display_line_start = 0x10;
        self.display_line_end = 0x100;
        self.display_depth = false;
    }

    fn gp1_display_mode(&mut self, ins: Instruction) {
        self.hres = ins.0 & 0b11;
        self.vres = ((ins.0 >> 2) & 0b1) != 0;
        self.display_depth = ((ins.0 >> 4) & 0b1) != 0;
        self.interlaced = ((ins.0 >> 5) & 0b1) != 0;
        self.hres |= (ins.0 >> 6) & 0b1;
        // Bit 7 not used on retail hardware
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
            Self::GP0_ADDR => self.gp0(Instruction(data)),
            Self::GP1_ADDR => self.gp1(Instruction(data)),
            _ => panic!("Invalid GPU address! {:#X}", address),
        }
    }
}

struct Instruction(u32);

impl Instruction {
    fn opcode(&self) -> u32 {
        self.0 >> 24
    }
}

