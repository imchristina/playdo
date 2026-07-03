use crate::bus::*;

pub struct Cpu {
    regs: [u32; 32],
    pc: u32,
    hi: u32,
    lo: u32,

    next_pc: u32, // Branch delay slot
    next_reg: (usize, u32), // Load delay slot
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: ROM_ADDR, // PS1 BIOS entry point
            hi: 0,
            lo: 0,

            next_pc: ROM_ADDR + 4,
            next_reg: (0, 0),
        }
    }

    pub fn step(&mut self, bus: &mut Bus) {
        let ins = Instruction(bus.read_u32(self.pc));

        self.execute(ins, bus);

        println!("Ins: {}, PC: {:#X}, Regs:{:X?} HI: {:#X}, LO: {:#X}", ins.op(), self.pc, self.regs, self.hi, self.lo);
    }

    pub fn execute(&mut self, ins: Instruction, bus: &mut Bus) {
        self.pc = self.next_pc;
        self.next_pc = self.pc + 4; // Default, branching instructions will overwrite

        self.set_reg(self.next_reg.0, self.next_reg.1);
        self.next_reg = (0,0);

        match ins.op() {
            00 => match ins.funct() {
                00 => self.op_sll(ins),
                _ => panic!("Unknown special instruction! Funct {}, raw {:#X} at address {:#X}", ins.funct(), ins.0, self.pc),
            }
            13 => self.op_ori(ins),
            15 => self.op_lui(ins),
            43 => self.op_sw(ins, bus),
            _ => panic!("Unknown instruction! OP {}, raw {:#X} at address {:#X}", ins.op(), ins.0, self.pc),
        }
    }

    fn op_sll(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.regs[ins.rt()] << ins.sa())
    }

    fn op_ori(&mut self, ins: Instruction) {
        self.set_reg(ins.rt(), self.regs[ins.rs()] | ins.imm_u32());
    }

    fn op_lui(&mut self, ins: Instruction) {
        self.set_reg(ins.rt(), (ins.imm_u32()) << 16);
    }

    fn op_sw(&mut self, ins: Instruction, bus: &mut Bus) {
        bus.write_u32(self.regs[ins.rs()] + ins.imm_se(), self.regs[ins.rt()]);
    }

    fn set_reg(&mut self, i: usize, val: u32) {
        self.regs[i] = val;
        self.regs[0] = 0; // Ensure register 0 is always 0
    }
}

#[derive(Clone, Copy)]
pub struct Instruction(u32);

impl Instruction {
    fn op(&self) -> u8 {
        (self.0 >> 26) as u8
    }
    fn rs(&self) -> usize {
        ((self.0 >> 21) & 0x1F) as usize
    }
    fn rt(&self) -> usize {
        ((self.0 >> 16) & 0x1F) as usize
    }
    fn rd(&self) -> usize {
        ((self.0 >> 11) & 0x1F) as usize
    }
    fn sa(&self) -> u8 {
        ((self.0 >> 6) & 0x1F) as u8
    }
    fn funct(&self) -> u8 {
        (self.0 & 0x3F) as u8
    }
    fn imm(&self) -> u16 {
        (self.0 & 0xFFFF) as u16
    }
    fn imm_u32(&self) -> u32 {
        (self.0 & 0xFFFF) as u32
    }
    pub fn imm_se(&self) -> u32 {
        (self.0 & 0xFFFF) as i16 as i32 as u32
    }
}
