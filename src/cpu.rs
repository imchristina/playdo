use crate::bus::Bus;

pub struct Cpu {
    regs: [u32; 32],
    pc: u32,
    hi: u32,
    lo: u32,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: 0xBFC00000, // PS1 BIOS entry point
            hi: 0,
            lo: 0,
        }
    }

    pub fn step(&mut self, bus: &mut Bus) {
        let ins = Instruction(bus.read_u32(self.pc));

        let branch = self.execute(ins, bus);

        println!("Ins: {}, PC: {:#X}, Regs:{:X?} HI: {:#X}, LO: {:#X}", ins.op(), self.pc, self.regs, self.hi, self.lo);

        if !branch {
            self.pc += 4;
        }
    }

    pub fn execute(&mut self, ins: Instruction, bus: &mut Bus) -> bool {
        match ins.op() {
            13 => self.op_ori(ins),
            15 => self.op_lui(ins),
            43 => self.op_sw(ins, bus),
            _ => panic!("Unknown instruction! {:#X} at address {:#X}", ins.0, self.pc),
        }

        false // No branch taken
    }

    fn op_ori(&mut self, ins: Instruction) {
        self.regs[ins.rt() as usize] = self.regs[ins.rs() as usize] | (ins.imm() as u32)
    }

    fn op_lui(&mut self, ins: Instruction) {
        self.regs[ins.rt() as usize] = (ins.imm() as u32) << 16;
    }

    fn op_sw(&mut self, ins: Instruction, bus: &mut Bus) {
        bus.write_u32(self.regs[ins.rs() as usize] + ins.imm_se(), self.regs[ins.rt() as usize]);
    }
}

#[derive(Clone, Copy)]
pub struct Instruction(u32);

impl Instruction {
    fn op(&self) -> u8 {
        (self.0 >> 26) as u8
    }
    fn rs(&self) -> u8 {
        ((self.0 >> 21) & 0x1F) as u8
    }
    fn rt(&self) -> u8 {
        ((self.0 >> 16) & 0x1F) as u8
    }
    fn rd(&self) -> u8 {
        ((self.0 >> 11) & 0x1F) as u8
    }
    fn shamt(&self) -> u8 {
        ((self.0 >> 6) & 0x1F) as u8
    }
    fn funct(&self) -> u8 {
        (self.0 & 0x1F) as u8
    }
    fn imm(&self) -> u16 {
        (self.0 & 0xFFFF) as u16
    }
    pub fn imm_se(&self) -> u32 {
        (self.0 & 0xFFFF) as i16 as i32 as u32
    }
}
