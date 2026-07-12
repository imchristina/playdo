use crate::bus::*;

pub const ENTRY_POINT: u32 = 0xBFC00000;

const COP0_REG_SR: usize = 12;

const COP0_SR_IEC: u32 = 1 << 0;
const COP0_SR_KUC: u32 = 1 << 1;
const COP0_SR_IEP: u32 = 1 << 2;
const COP0_SR_KUP: u32 = 1 << 3;
const COP0_SR_IEO: u32 = 1 << 4;
const COP0_SR_KUO: u32 = 1 << 5;
const COP0_SR_CUR_SHIFT: u32 = 0;
const COP0_SR_CUR_MASK: u32 = 3 << COP0_SR_CUR_SHIFT;
const COP0_SR_PREV_SHIFT: u32 = 2;
const COP0_SR_PREV_MASK: u32 = 3 << COP0_SR_PREV_SHIFT;
const COP0_SR_OLD_SHIFT: u32 = 4;
const COP0_SR_OLD_MASK: u32 = 3 << COP0_SR_OLD_SHIFT;

pub struct Cpu {
    regs: [u32; 32],
    pc: u32,
    hi: u32,
    lo: u32,

    branch_delay: u32, // Branch delay slot
    load_delay: [RegWrite; 2], // Load delay slot, last member is oldest/to be commited

    cop0_regs: [u32; 32],
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: ENTRY_POINT, // PS1 BIOS entry point
            hi: 0,
            lo: 0,

            branch_delay: ENTRY_POINT + 4,
            load_delay: [RegWrite::default(); 2],

            cop0_regs: [0; 32],
        }
    }

    pub fn step(&mut self, bus: &mut Bus) {
        let ins = Instruction(bus.read_u32(self.pc));

        let pc_debug = self.pc; // Store PC before delay slot is active

        self.execute(ins, bus);

        println!("OP: {}, PC: {:#X}, Regs:{:X?} HI: {:#X}, LO: {:#X}", ins.op(), pc_debug, self.regs, self.hi, self.lo);
    }

    pub fn execute(&mut self, ins: Instruction, bus: &mut Bus) {
        self.pc = self.branch_delay;
        self.branch_delay = self.pc + 4; // Default, branching instructions will overwrite

        self.set_reg(self.load_delay[1].addr, self.load_delay[1].data); // Commit load delay
        self.load_delay[1] = self.load_delay[0];
        self.load_delay[0] = RegWrite::default();

        match ins.op() {
            00 => match ins.funct() {
                00 => self.op_sll(ins),
                08 => self.op_jr(ins),
                09 => self.op_jalr(ins),
                32 => self.op_add(ins),
                33 => self.op_addu(ins),
                36 => self.op_and(ins),
                37 => self.op_or(ins),
                43 => self.op_sltu(ins),
                _ => panic!("Unknown special instruction! Funct {}, raw {:#b} at address {:#X}", ins.funct(), ins.0, self.pc),
            }
            02 => self.op_j(ins),
            03 => self.op_jal(ins),
            04 => self.op_beq(ins),
            05 => self.op_bne(ins),
            08 => self.op_addi(ins),
            09 => self.op_addiu(ins),
            12 => self.op_andi(ins),
            13 => self.op_ori(ins),
            15 => self.op_lui(ins),
            16 => match ins.rs() {
                0b00000 => self.cop0_op_mfc(ins),
                0b00100 => self.cop0_op_mtc(ins),
                0b10000 => self.cop0_op_rfe(),
                _ => panic!("Unknown COP0 instruction! OP (RS) {}, raw {:#b} at address {:#X}", ins.rs(), ins.0, self.pc)
            }
            32 => self.op_lb(ins, bus),
            35 => self.op_lw(ins, bus),
            40 => self.op_sb(ins, bus),
            41 => self.op_sh(ins, bus),
            43 => self.op_sw(ins, bus),
            _ => panic!("Unknown instruction! OP {}, raw {:#b} at address {:#X}", ins.op(), ins.0, self.pc),
        }
    }

    fn op_sll(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.regs[ins.rt()] << ins.sa());
    }

    fn op_jr(&mut self, ins: Instruction) {
        self.branch_delay = self.regs[ins.rs()];
    }

    fn op_jalr(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.branch_delay);
        self.branch_delay = self.regs[ins.rs()];
    }

    fn op_add(&mut self, ins: Instruction) { // TODO overflow trap
        self.set_reg(ins.rd(), self.regs[ins.rs()] + self.regs[ins.rt()]);
    }

    fn op_addu(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.regs[ins.rs()] + self.regs[ins.rt()]);
    }

    fn op_and(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.regs[ins.rs()] & self.regs[ins.rt()]);
    }

    fn op_or(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.regs[ins.rs()] | self.regs[ins.rt()]);
    }

    fn op_sltu(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), (self.regs[ins.rs()] < self.regs[ins.rt()]) as u32);
    }

    fn op_j(&mut self, ins: Instruction) {
        self.branch_delay = (self.branch_delay & 0xF0000000) | ins.target() << 2;
    }

    fn op_jal(&mut self, ins: Instruction) {
        self.set_reg(31, self.branch_delay);
        self.branch_delay = (self.branch_delay & 0xF0000000) | ins.target() << 2;
    }

    fn op_beq(&mut self, ins: Instruction) {
        if self.regs[ins.rs()] == self.regs[ins.rt()] {
            self.branch_delay = self.pc + (ins.imm_se() << 2);
        }
    }

    fn op_bne(&mut self, ins: Instruction) {
        if self.regs[ins.rs()] != self.regs[ins.rt()] {
            self.branch_delay = self.pc + (ins.imm_se() << 2);
        }
    }

    fn op_addi(&mut self, ins: Instruction) { // TODO overflow trap
        self.set_reg(ins.rt(), ins.imm_se() + self.regs[ins.rs()]);
    }

    fn op_addiu(&mut self, ins: Instruction) {
        self.set_reg(ins.rt(), ins.imm_se() + self.regs[ins.rs()]);
    }

    fn op_andi(&mut self, ins: Instruction) {
        self.set_reg(ins.rt(), self.regs[ins.rs()] & ins.imm_u32());
    }

    fn op_ori(&mut self, ins: Instruction) {
        self.set_reg(ins.rt(), self.regs[ins.rs()] | ins.imm_u32());
    }

    fn op_lui(&mut self, ins: Instruction) {
        self.set_reg(ins.rt(), (ins.imm_u32()) << 16);
    }

    fn op_lb(&mut self, ins: Instruction, bus: &mut Bus) {
        self.set_reg_delay(ins.rt(), bus.read_u8(self.regs[ins.rs()] + ins.imm_se()) as i32 as u32);
    }

    fn op_lw(&mut self, ins: Instruction, bus: &mut Bus) {
        self.set_reg_delay(ins.rt(), bus.read_u32(self.regs[ins.rs()] + ins.imm_se()));
    }

    fn op_sb(&mut self, ins: Instruction, bus: &mut Bus) {
        bus.write_u8(self.regs[ins.rs()] + ins.imm_se(), self.regs[ins.rt()] as u8);
    }

    fn op_sh(&mut self, ins: Instruction, bus: &mut Bus) {
        bus.write_u16(self.regs[ins.rs()] + ins.imm_se(), self.regs[ins.rt()] as u16);
    }

    fn op_sw(&mut self, ins: Instruction, bus: &mut Bus) {
        bus.write_u32(self.regs[ins.rs()] + ins.imm_se(), self.regs[ins.rt()]);
    }

    fn cop0_op_mfc(&mut self, ins: Instruction) {
        self.set_reg_delay(ins.rt(), self.cop0_regs[ins.rd()]);
    }

    fn cop0_op_mtc(&mut self, ins: Instruction) {
        self.cop0_regs[ins.rd()] = self.regs[ins.rt()];
    }

    fn cop0_op_rfe(&mut self) {
        let sr = self.cop0_regs[COP0_REG_SR];
        let prev = (sr & COP0_SR_PREV_MASK) >> COP0_SR_PREV_SHIFT;
        let old = (sr & COP0_SR_OLD_MASK) >> COP0_SR_OLD_SHIFT;

        let mut new_sr = sr & !(COP0_SR_CUR_MASK | COP0_SR_PREV_MASK);
        new_sr |= prev | old;

        self.cop0_regs[COP0_REG_SR] = new_sr;
    }

    fn set_reg(&mut self, i: usize, val: u32) {
        self.regs[i] = val;
        self.regs[0] = 0; // Ensure register 0 is always 0
    }

    fn set_reg_delay(&mut self, i: usize, val: u32) {
        self.load_delay[0].addr = i;
        self.load_delay[0].data = val;
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
        (self.0 & 0xFFFF)
    }
    fn imm_se(&self) -> u32 {
        (self.0 & 0xFFFF) as i16 as i32 as u32
    }
    fn target(&self) -> u32 {
        (self.0 & 0x3ffffff)
    }
}

#[derive(Clone, Copy, Default)]
pub struct RegWrite {
    addr: usize,
    data: u32,
}
