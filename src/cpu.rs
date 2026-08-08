use crate::bus::*;

pub const ENTRY_POINT:          u32 = 0xBFC00000;
pub const EXCEPTION_VECTOR:      u32 = 0x80000080;
pub const EXCEPTION_VECTOR_ALT:  u32 = 0xBFC00180;

const COP0_REG_SR:      usize = 12;
const COP0_REG_CAUSE:   usize = 13;
const COP0_REG_EPC:     usize = 14;

const COP0_SR_IEC: u32 = 1 << 0;
const COP0_SR_KUC: u32 = 1 << 1;
const COP0_SR_IEP: u32 = 1 << 2;
const COP0_SR_KUP: u32 = 1 << 3;
const COP0_SR_IEO: u32 = 1 << 4;
const COP0_SR_KUO: u32 = 1 << 5;
const COP0_SR_IM2: u32 = 1 << 10;
const COP0_SR_ISC: u32 = 1 << 16;
const COP0_SR_BEV: u32 = 1 << 22;

const COP0_CAUSE_EXECCODE_SHIFT:    u32 = 2;
const COP0_CAUSE_EXECCODE_MASK:     u32 = 0b11111 << COP0_CAUSE_EXECCODE_SHIFT;
const COP0_CAUSE_IP_SHIFT:          u32 = 8;
const COP0_CAUSE_IP_MASK:           u32 = 0b11111111 << COP0_CAUSE_IP_SHIFT;
const COP0_CAUSE_IP2:              u32 = 1 << 10;
const COP0_CAUSE_BD:                u32 = 1 << 31;

const COP0_EXECCODE_INT:        u32 = 0;
const COP0_EXECCODE_SYSCALL:    u32 = 8;
const COP0_EXECCODE_OV:          u32 = 12;

pub struct Cpu {
    pub regs: [u32; 32],
    pub pc: u32,
    pub hi: u32,
    pub lo: u32,

    pub branch_delay: u32, // Branch delay slot
    pub load_delay: [RegWrite; 2], // Load delay slot, last member is oldest/to be commited
    pub pc_ins: u32, // PC of the currently executing instruction, for exceptions

    pub cop0_regs: [u32; 32],
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
            pc_ins: ENTRY_POINT,

            cop0_regs: [0; 32],
        }
    }

    pub fn step(&mut self, bus: &mut Bus) {
        // Interrupts
        self.cop0_regs[COP0_REG_CAUSE] &= !COP0_CAUSE_IP2;
        if (bus.interrupt.i_stat & bus.interrupt.i_mask) != 0 {
            self.cop0_regs[COP0_REG_CAUSE] |= COP0_CAUSE_IP2;

            let sr = self.cop0_regs[COP0_REG_SR];
            if ((sr & COP0_SR_IEC) != 0) && ((sr & COP0_SR_IM2) != 0) {
                self.exception(COP0_EXECCODE_INT);
            }
        }

        let ins = Instruction(bus.read_u32(self.pc));

        self.execute(ins, bus);

        self.stdio_hook();
    }

    pub fn execute(&mut self, ins: Instruction, bus: &mut Bus) {
        self.pc_ins = self.pc;
        self.pc = self.branch_delay;
        self.branch_delay = self.pc + 4; // Default, branching instructions will overwrite

        self.set_reg(self.load_delay[1].addr, self.load_delay[1].data); // Commit load delay
        self.load_delay[1] = self.load_delay[0];
        self.load_delay[0] = RegWrite::default();

        match ins.op() {
            00 => match ins.funct() {
                00 => self.op_sll(ins),
                02 => self.op_srl(ins),
                03 => self.op_sra(ins),
                04 => self.op_sllv(ins),
                06 => self.op_srlv(ins),
                07 => self.op_srav(ins),
                08 => self.op_jr(ins),
                09 => self.op_jalr(ins),
                12 => self.op_syscall(),
                16 => self.op_mfhi(ins),
                17 => self.op_mthi(ins),
                18 => self.op_mflo(ins),
                19 => self.op_mtlo(ins),
                //24 => self.op_mult(ins),
                25 => self.op_multu(ins),
                26 => self.op_div(ins),
                27 => self.op_divu(ins),
                32 => self.op_add(ins),
                33 => self.op_addu(ins),
                34 => self.op_sub(ins),
                35 => self.op_subu(ins),
                36 => self.op_and(ins),
                37 => self.op_or(ins),
                38 => self.op_xor(ins),
                39 => self.op_nor(ins),
                42 => self.op_slt(ins),
                43 => self.op_sltu(ins),
                _ => panic!("Unknown special instruction! Funct {}, raw {:#b} at address {:#X}", ins.funct(), ins.0, self.pc),
            }
            01 => match ins.rt() {
                00 => self.op_bltz(ins),
                01 => self.op_bgez(ins),
                _ => panic!("Unknown branch instruction! RT {}, raw {:#b} at address {:#X}", ins.rt(), ins.0, self.pc),
            }
            02 => self.op_j(ins),
            03 => self.op_jal(ins),
            04 => self.op_beq(ins),
            05 => self.op_bne(ins),
            06 => self.op_blez(ins),
            07 => self.op_bgtz(ins),
            08 => self.op_addi(ins),
            09 => self.op_addiu(ins),
            10 => self.op_slti(ins),
            11 => self.op_sltiu(ins),
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
            33 => self.op_lh(ins, bus),
            35 => self.op_lw(ins, bus),
            36 => self.op_lbu(ins, bus),
            37 => self.op_lhu(ins, bus),
            40 => self.op_sb(ins, bus),
            41 => self.op_sh(ins, bus),
            43 => self.op_sw(ins, bus),
            _ => panic!("Unknown instruction! OP {}, raw {:#b} at address {:#X}", ins.op(), ins.0, self.pc),
        }
    }

    fn op_sll(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.regs[ins.rt()] << ins.sa());
    }

    fn op_srl(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.regs[ins.rt()] >> ins.sa());
    }

    fn op_sra(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), (self.regs[ins.rt()] as i32 >> ins.sa()) as u32);
    }

    fn op_sllv(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.regs[ins.rt()] << (self.regs[ins.rs()] & 0x1F));
    }

    fn op_srlv(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.regs[ins.rt()] >> (self.regs[ins.rs()] & 0x1F));
    }

    fn op_srav(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), ((self.regs[ins.rt()] as i32) >> (self.regs[ins.rs()] & 0x1F)) as u32);
    }

    fn op_jr(&mut self, ins: Instruction) {
        self.branch_delay = self.regs[ins.rs()];
    }

    fn op_jalr(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.branch_delay);
        self.branch_delay = self.regs[ins.rs()];
    }

    fn op_syscall(&mut self) {
        self.exception(COP0_EXECCODE_SYSCALL);
    }

    fn op_mfhi(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.hi);
    }

    fn op_mflo(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.lo);
    }

    fn op_mthi(&mut self, ins: Instruction) {
        self.hi = self.regs[ins.rs()];
    }

    fn op_mtlo(&mut self, ins: Instruction) {
        self.lo = self.regs[ins.rs()];
    }

    fn op_multu(&mut self, ins: Instruction) {
        let result = self.regs[ins.rs()] as u64 * self.regs[ins.rt()] as u64;

        self.lo = result as u32;
        self.hi = (result >> 32) as u32;
    }

    fn op_div(&mut self, ins: Instruction) {
        self.lo = (self.regs[ins.rs()] as i32 / self.regs[ins.rt()] as i32) as u32;
        self.hi = (self.regs[ins.rs()] as i32 % self.regs[ins.rt()] as i32) as u32;
    }

    fn op_divu(&mut self, ins: Instruction) {
        self.lo = self.regs[ins.rs()] / self.regs[ins.rt()];
        self.hi = self.regs[ins.rs()] % self.regs[ins.rt()];
    }

    fn op_add(&mut self, ins: Instruction) {
        let result = i32::checked_add(self.regs[ins.rs()] as i32, self.regs[ins.rt()] as i32);
        match result {
            Some(value) => self.set_reg(ins.rd(), value as u32),
            None => self.exception(COP0_EXECCODE_OV),
        }
    }

    fn op_addu(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.regs[ins.rs()] + self.regs[ins.rt()]);
    }

    fn op_sub(&mut self, ins: Instruction) {
        let result = i32::checked_sub(self.regs[ins.rs()] as i32, self.regs[ins.rt()] as i32);
        match result {
            Some(value) => self.set_reg(ins.rd(), value as u32),
            None => self.exception(COP0_EXECCODE_OV),
        }
    }

    fn op_subu(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.regs[ins.rs()] - self.regs[ins.rt()]);
    }

    fn op_and(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.regs[ins.rs()] & self.regs[ins.rt()]);
    }

    fn op_or(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.regs[ins.rs()] | self.regs[ins.rt()]);
    }

    fn op_xor(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), self.regs[ins.rs()] ^ self.regs[ins.rt()]);
    }

    fn op_nor(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), !(self.regs[ins.rs()] | self.regs[ins.rt()]));
    }

    fn op_slt(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), ((self.regs[ins.rs()] as i32) < (self.regs[ins.rt()] as i32)) as u32);
    }

    fn op_sltu(&mut self, ins: Instruction) {
        self.set_reg(ins.rd(), (self.regs[ins.rs()] < self.regs[ins.rt()]) as u32);
    }

    fn op_bltz(&mut self, ins: Instruction) {
        if (self.regs[ins.rs()] & (1 << 31)) != 0 {
            self.branch_delay = self.pc + (ins.imm_se() << 2);
        }
    }

    fn op_bgez(&mut self, ins: Instruction) {
        if (self.regs[ins.rs()] & (1 << 31)) == 0 {
            self.branch_delay = self.pc + (ins.imm_se() << 2);
        }
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

    fn op_blez(&mut self, ins: Instruction) {
        if (self.regs[ins.rs()] as i32) <= 0 {
            self.branch_delay = self.pc + (ins.imm_se() << 2);
        }
    }

    fn op_bgtz(&mut self, ins: Instruction) {
        if (self.regs[ins.rs()] as i32) > 0 {
            self.branch_delay = self.pc + (ins.imm_se() << 2);
        }
    }

    fn op_bne(&mut self, ins: Instruction) {
        if self.regs[ins.rs()] != self.regs[ins.rt()] {
            self.branch_delay = self.pc + (ins.imm_se() << 2);
        }
    }

    fn op_addi(&mut self, ins: Instruction) {
        let result = i32::checked_add(ins.imm_se() as i32, self.regs[ins.rs()] as i32);
        match result {
            Some(value) => self.set_reg(ins.rt(), value as u32),
            None => self.exception(COP0_EXECCODE_OV),
        }
    }

    fn op_addiu(&mut self, ins: Instruction) {
        self.set_reg(ins.rt(), ins.imm_se() + self.regs[ins.rs()]);
    }

    fn op_slti(&mut self, ins: Instruction) {
        self.set_reg(ins.rt(), ((self.regs[ins.rs()] as i32) < (ins.imm_se() as i32)) as u32);
    }

    fn op_sltiu(&mut self, ins: Instruction) {
        self.set_reg(ins.rt(), (self.regs[ins.rs()] < ins.imm_se()) as u32);
    }

    fn op_andi(&mut self, ins: Instruction) {
        self.set_reg(ins.rt(), self.regs[ins.rs()] & ins.imm());
    }

    fn op_ori(&mut self, ins: Instruction) {
        self.set_reg(ins.rt(), self.regs[ins.rs()] | ins.imm());
    }

    fn op_lui(&mut self, ins: Instruction) {
        self.set_reg(ins.rt(), (ins.imm()) << 16);
    }

    fn op_lb(&mut self, ins: Instruction, bus: &mut Bus) {
        self.set_reg_delay(ins.rt(), bus.read_u8(self.regs[ins.rs()] + ins.imm_se()) as i8 as u32);
    }

    fn op_lh(&mut self, ins: Instruction, bus: &mut Bus) {
        self.set_reg_delay(ins.rt(), bus.read_u16(self.regs[ins.rs()] + ins.imm_se()) as i16 as u32);
    }

    fn op_lw(&mut self, ins: Instruction, bus: &mut Bus) {
        self.set_reg_delay(ins.rt(), bus.read_u32(self.regs[ins.rs()] + ins.imm_se()));
    }

    fn op_lbu(&mut self, ins: Instruction, bus: &mut Bus) {
        self.set_reg_delay(ins.rt(), bus.read_u8(self.regs[ins.rs()] + ins.imm_se()) as u32);
    }

    fn op_lhu(&mut self, ins: Instruction, bus: &mut Bus) {
        self.set_reg_delay(ins.rt(), bus.read_u16(self.regs[ins.rs()] + ins.imm_se()) as u32);
    }

    fn op_sb(&mut self, ins: Instruction, bus: &mut Bus) {
        if (self.cop0_regs[COP0_REG_SR] & COP0_SR_ISC) == 0 {
            bus.write_u8(self.regs[ins.rs()] + ins.imm_se(), self.regs[ins.rt()] as u8);
        }
    }

    fn op_sh(&mut self, ins: Instruction, bus: &mut Bus) {
        if (self.cop0_regs[COP0_REG_SR] & COP0_SR_ISC) == 0 {
            bus.write_u16(self.regs[ins.rs()] + ins.imm_se(), self.regs[ins.rt()] as u16);
        }
    }

    fn op_sw(&mut self, ins: Instruction, bus: &mut Bus) {
        if (self.cop0_regs[COP0_REG_SR] & COP0_SR_ISC) == 0 {
            bus.write_u32(self.regs[ins.rs()] + ins.imm_se(), self.regs[ins.rt()]);
        }
    }

    fn cop0_op_mfc(&mut self, ins: Instruction) {
        self.set_reg_delay(ins.rt(), self.cop0_regs[ins.rd()]);
    }

    fn cop0_op_mtc(&mut self, ins: Instruction) {
        self.cop0_regs[ins.rd()] = self.regs[ins.rt()];
    }

    fn cop0_op_rfe(&mut self) {
        let sr = self.cop0_regs[COP0_REG_SR];

        let mode = (sr >> 2) & 0x0F;

        self.cop0_regs[COP0_REG_SR] = (sr & !0x0F) | mode;
    }

    fn set_reg(&mut self, i: usize, val: u32) {
        self.regs[i] = val;
        self.regs[0] = 0; // Ensure register 0 is always 0
    }

    fn set_reg_delay(&mut self, i: usize, val: u32) {
        self.load_delay[0].addr = i;
        self.load_delay[0].data = val;
    }

    fn exception(&mut self, exccode: u32) {
        let mut sr = self.cop0_regs[COP0_REG_SR];
        let mut cause = self.cop0_regs[COP0_REG_CAUSE];
        let epc;

        // Check if in a delay slot
        if self.pc != self.pc_ins + 4 {
            cause |= COP0_CAUSE_BD;
            epc = self.pc_ins - 4;
        } else {
            cause &= !COP0_CAUSE_BD;
            epc = self.pc_ins;
        }

        cause = (cause & !COP0_CAUSE_EXECCODE_MASK) | (exccode << COP0_CAUSE_EXECCODE_SHIFT);

        let mode = sr & 0x3F;
        sr = (sr & !0x3F) | ((mode << 2) & 0x3F);

        let vector;
        if (sr & COP0_SR_BEV) == 0 {
            vector = EXCEPTION_VECTOR;
        } else {
            vector = EXCEPTION_VECTOR_ALT;
        }

        self.pc = vector;
        self.branch_delay = vector + 4;

        self.cop0_regs[COP0_REG_SR] = sr;
        self.cop0_regs[COP0_REG_CAUSE] = cause;
        self.cop0_regs[COP0_REG_EPC] = epc;
    }

    fn stdio_hook(&self) {
        if (self.pc == 0xA0) || (self.pc == 0xB0) {
            let func = self.regs[9];
            if (func == 0x3C) || (func == 0x3D) {
                print!("{}", self.regs[4] as u8 as char)
            }
        }
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
    fn imm(&self) -> u32 {
        self.0 & 0xFFFF
    }
    fn imm_se(&self) -> u32 {
        (self.0 & 0xFFFF) as i16 as i32 as u32
    }
    fn target(&self) -> u32 {
        self.0 & 0x3ffffff
    }
}

#[derive(Clone, Copy, Default)]
pub struct RegWrite {
    addr: usize,
    data: u32,
}
