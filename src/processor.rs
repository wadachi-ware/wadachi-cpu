use crate::csr::Csr;
use crate::decode::{decode, BType, IType, Instruction, JType, RType, SType, UType};
use crate::exception::Exception;
use crate::memory::Memory;

/// Priviledge level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    User = 0b00,
    Supervisor = 0b01,
    Machine = 0b11,
}

pub struct Processor {
    pub pc: u32,
    pub(crate) regs: [u32; 32],
    pub(crate) csr: Csr,
    mem: Box<dyn Memory>,
    mode: Mode,
    // Used to determine if the pc should be incremented.
    has_jumped: bool,
}

impl Processor {
    /// Instruction execution starts from the `pc`.
    pub fn new(memory: Box<dyn Memory>) -> Self {
        Self {
            pc: 0,
            regs: [0; 32],
            csr: Csr::default(),
            mem: memory,
            mode: Mode::Machine,
            has_jumped: false,
        }
    }

    /// Set program counter to start instruction execution.
    pub fn set_pc(&mut self, pc: u32) {
        if pc % 4 != 0 {
            // If this rule is broken, instruction execution will never be done properly.
            // And this is not during instruction execution, so returning `Exception` is
            // inappropriate.
            panic!("Instruction address must be aligned to a 4byte boundary");
        }
        self.pc = pc;
    }

    /// Load a program, which is an array of `u32` integer, in the `address`.
    pub fn load(&mut self, address: u32, program: Vec<u32>) {
        if address % 4 != 0 {
            panic!("Instruction address must be aligned to a 4byte boundary");
        }
        for (index, instruction) in program.iter().enumerate() {
            self.mem
                .write_inst(address as usize + index * 4, *instruction);
        }
    }

    /// Execute the program stored in the memory.
    pub fn execute(&mut self) {
        loop {
            if self.tick().is_err() {
                // We have nothing to do with exception, stop the loop for now.
                break;
            }
        }
    }

    /// Read the register value at index `idx`.
    fn read_reg(&self, idx: usize) -> u32 {
        if idx == 0 {
            0
        } else {
            self.regs[idx]
        }
    }

    /// Write value to the register at index `idx`.
    fn write_reg(&mut self, idx: usize, val: u32) {
        if idx != 0 {
            self.regs[idx] = val;
        }
    }

    /// Read the CSR value at `addr`.
    /// `addr` is `u16` because immediate of CSR instruction is decoded as `u16`.
    fn read_csr(&self, addr: u16) -> Result<u32, Exception> {
        self.csr.read(addr as usize, self.mode)
    }

    /// Write `value` to the CSR at `addr`.
    fn write_csr(&mut self, addr: u16, value: u32) -> Result<(), Exception> {
        self.csr.write(addr as usize, value, self.mode)
    }

    /// Read an instruction from current program counter and execute it.
    pub fn tick(&mut self) -> Result<(), Exception> {
        if self.pc + 4 > self.mem.len() as u32 {
            return Err(Exception::InstructionAccessFault);
        }

        let raw_inst = self.mem.read_inst(self.pc as usize);
        match decode(raw_inst)? {
            // R-Type
            Instruction::Add(args) => self.inst_add(&args),
            Instruction::Sub(args) => self.inst_sub(&args),
            Instruction::Sll(args) => self.inst_sll(&args),
            Instruction::Slt(args) => self.inst_slt(&args),
            Instruction::Sltu(args) => self.inst_sltu(&args),
            Instruction::Xor(args) => self.inst_xor(&args),
            Instruction::Srl(args) => self.inst_srl(&args),
            Instruction::Sra(args) => self.inst_sra(&args),
            Instruction::Or(args) => self.inst_or(&args),
            Instruction::And(args) => self.inst_and(&args),

            // I-Type
            Instruction::Jalr(args) => self.inst_jalr(&args)?,
            Instruction::Addi(args) => self.inst_addi(&args),
            Instruction::Slli(args) => self.inst_slli(&args),
            Instruction::Slti(args) => self.inst_slti(&args),
            Instruction::Sltiu(args) => self.inst_sltiu(&args),
            Instruction::Xori(args) => self.inst_xori(&args),
            Instruction::Srli(args) => self.inst_srli(&args),
            Instruction::Srai(args) => self.inst_srai(&args),
            Instruction::Ori(args) => self.inst_ori(&args),
            Instruction::Andi(args) => self.inst_andi(&args),
            Instruction::Lb(args) => self.inst_lb(&args),
            Instruction::Lh(args) => self.inst_lh(&args),
            Instruction::Lw(args) => self.inst_lw(&args),
            Instruction::Lbu(args) => self.inst_lbu(&args),
            Instruction::Lhu(args) => self.inst_lhu(&args),
            Instruction::Csrrw(args) => self.inst_csrrw(&args)?,
            Instruction::Csrrs(args) => self.inst_csrrs(&args)?,
            Instruction::Csrrc(args) => self.inst_csrrc(&args)?,
            Instruction::Csrrwi(args) => self.inst_csrrwi(&args)?,
            Instruction::Csrrsi(args) => self.inst_csrrsi(&args)?,
            Instruction::Csrrci(args) => self.inst_csrrci(&args)?,

            // S-Type
            Instruction::Sb(args) => self.inst_sb(&args),
            Instruction::Sh(args) => self.inst_sh(&args),
            Instruction::Sw(args) => self.inst_sw(&args),

            // B-Type
            Instruction::Beq(args) => self.inst_beq(&args)?,
            Instruction::Bne(args) => self.inst_bne(&args)?,
            Instruction::Blt(args) => self.inst_blt(&args)?,
            Instruction::Bge(args) => self.inst_bge(&args)?,
            Instruction::Bltu(args) => self.inst_bltu(&args)?,
            Instruction::Bgeu(args) => self.inst_bgeu(&args)?,

            // U-Type
            Instruction::Auipc(args) => self.inst_auipc(&args),
            Instruction::Lui(args) => self.inst_lui(&args),

            // J-Type
            Instruction::Jal(args) => self.inst_jal(&args)?,
        }

        // If no jump occured, increment pc.
        if !self.has_jumped {
            self.pc += 4;
        }
        self.has_jumped = false;

        Ok(())
    }
}

impl Processor {
    const fn sign_extend(val: u16) -> u32 {
        if val & 0x800 != 0 {
            (val as u32) | 0xfffff000
        } else {
            val as u32
        }
    }

    // Sign extend given integer with 20bit.
    const fn sign_extend_20bit(value: u32) -> i32 {
        if value & 0xfff80000 != 0 {
            (value | 0xfff00000) as i32
        } else {
            value as i32
        }
    }

    fn inst_add(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1);
        let rv = self.read_reg(args.rs2);
        let v = lv.wrapping_add(rv);
        self.write_reg(args.rd, v);
    }

    fn inst_sub(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1);
        let rv = self.read_reg(args.rs2);
        let v = lv.wrapping_sub(rv);
        self.write_reg(args.rd, v);
    }

    fn inst_sll(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1);
        let rv = self.read_reg(args.rs2);
        let v = lv << rv;
        self.write_reg(args.rd, v);
    }

    fn inst_slt(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1) as i32;
        let rv = self.read_reg(args.rs2) as i32;
        let v = (lv < rv) as u32;
        self.write_reg(args.rd, v);
    }

    fn inst_sltu(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1);
        let rv = self.read_reg(args.rs2);
        let v = (lv < rv) as u32;
        self.write_reg(args.rd, v);
    }

    fn inst_xor(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1);
        let rv = self.read_reg(args.rs2);
        let v = lv ^ rv;
        self.write_reg(args.rd, v);
    }

    fn inst_srl(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1);
        let rv = self.read_reg(args.rs2);
        let v = lv >> rv;
        self.write_reg(args.rd, v);
    }

    fn inst_sra(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1) as i32;
        let rv = self.read_reg(args.rs2);
        let v = (lv >> rv) as u32;
        self.write_reg(args.rd, v);
    }

    fn inst_or(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1);
        let rv = self.read_reg(args.rs2);
        let v = lv | rv;
        self.write_reg(args.rd, v);
    }

    fn inst_and(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1);
        let rv = self.read_reg(args.rs2);
        let v = lv & rv;
        self.write_reg(args.rd, v);
    }

    fn inst_jalr(&mut self, args: &IType) -> Result<(), Exception> {
        let lv = self.read_reg(args.rs1);
        let rv = Self::sign_extend(args.imm);
        let new_pc = (lv + rv) & 0xffff_fffe;
        if new_pc % 4 != 0 {
            return Err(Exception::InstructionAddressMisaligned);
        }
        self.write_reg(args.rd, self.pc + 4);
        self.set_pc(new_pc);
        self.has_jumped = true;
        Ok(())
    }

    fn inst_addi(&mut self, args: &IType) {
        let lv = self.read_reg(args.rs1) as i32;
        let rv = Self::sign_extend(args.imm) as i32;
        let v = (lv + rv) as u32;
        self.write_reg(args.rd, v);
    }

    fn inst_slli(&mut self, args: &IType) {
        let lv = self.read_reg(args.rs1);
        let rv = args.imm & 0x1f;
        let v = lv << rv;
        self.write_reg(args.rd, v);
    }

    fn inst_slti(&mut self, args: &IType) {
        let lv = self.read_reg(args.rs1) as i32;
        let rv = Self::sign_extend(args.imm) as i32;
        let v = (lv < rv) as u32;
        self.write_reg(args.rd, v);
    }

    fn inst_sltiu(&mut self, args: &IType) {
        let lv = self.read_reg(args.rs1);
        let rv = Self::sign_extend(args.imm);
        let v = (lv < rv) as u32;
        self.write_reg(args.rd, v);
    }

    fn inst_xori(&mut self, args: &IType) {
        let lv = self.read_reg(args.rs1);
        let rv = Self::sign_extend(args.imm);
        let v = lv ^ rv;
        self.write_reg(args.rd, v);
    }

    fn inst_srli(&mut self, args: &IType) {
        let lv = self.read_reg(args.rs1);
        let rv = args.imm & 0x1f;
        let v = (lv >> rv) as u32;
        self.write_reg(args.rd, v);
    }

    fn inst_srai(&mut self, args: &IType) {
        let lv = self.read_reg(args.rs1) as i32;
        let rv = args.imm & 0x1f;
        let v = (lv >> rv) as u32;
        self.write_reg(args.rd, v);
    }

    fn inst_ori(&mut self, args: &IType) {
        let lv = self.read_reg(args.rs1);
        let rv = Self::sign_extend(args.imm);
        let v = lv | rv;
        self.write_reg(args.rd, v);
    }

    fn inst_andi(&mut self, args: &IType) {
        let lv = self.read_reg(args.rs1);
        let rv = Self::sign_extend(args.imm);
        let v = lv & rv;
        self.write_reg(args.rd, v);
    }

    fn inst_lb(&mut self, args: &IType) {
        let lv = self.read_reg(args.rs1);
        let rv = Self::sign_extend(args.imm);
        let addr = (lv + rv) as usize;
        let v = (self.mem.read_byte(addr) as i8) as u32;
        self.write_reg(args.rd, v);
    }

    fn inst_lh(&mut self, args: &IType) {
        let lv = self.read_reg(args.rs1);
        let rv = Self::sign_extend(args.imm);
        let addr = (lv + rv) as usize;
        let v = (self.mem.read_halfword(addr) as i16) as u32;
        self.write_reg(args.rd, v);
    }

    fn inst_lw(&mut self, args: &IType) {
        let lv = self.read_reg(args.rs1);
        let rv = Self::sign_extend(args.imm);
        let addr = (lv + rv) as usize;
        let v = self.mem.read_word(addr);
        self.write_reg(args.rd, v);
    }

    fn inst_lbu(&mut self, args: &IType) {
        let lv = self.read_reg(args.rs1);
        let rv = Self::sign_extend(args.imm);
        let addr = (lv + rv) as usize;
        let v = self.mem.read_byte(addr) as u32;
        self.write_reg(args.rd, v);
    }

    fn inst_lhu(&mut self, args: &IType) {
        let lv = self.read_reg(args.rs1);
        let rv = Self::sign_extend(args.imm);
        let addr = (lv + rv) as usize;
        let v = self.mem.read_halfword(addr) as u32;
        self.write_reg(args.rd, v);
    }

    fn inst_csrrw(&mut self, args: &IType) -> Result<(), Exception> {
        let old_csr = self.read_csr(args.imm)?;
        self.write_reg(args.rd, old_csr);
        let value = self.read_reg(args.rs1);
        self.write_csr(args.imm, value)?;
        Ok(())
    }

    fn inst_csrrs(&mut self, args: &IType) -> Result<(), Exception> {
        let old_csr = self.read_csr(args.imm)?;
        self.write_reg(args.rd, old_csr);
        let value = self.read_reg(args.rs1);
        self.write_csr(args.imm, old_csr | value)?;
        Ok(())
    }

    fn inst_csrrc(&mut self, args: &IType) -> Result<(), Exception> {
        let old_csr = self.read_csr(args.imm)?;
        self.write_reg(args.rd, old_csr);
        let value = self.read_reg(args.rs1);
        self.write_csr(args.imm, old_csr & !value)?;
        Ok(())
    }

    fn inst_csrrwi(&mut self, args: &IType) -> Result<(), Exception> {
        let old_csr = self.read_csr(args.imm)?;
        self.write_reg(args.rd, old_csr);
        // `rs1` is treated as immediate.
        self.write_csr(args.imm, args.rs1 as u32)?;
        Ok(())
    }

    fn inst_csrrsi(&mut self, args: &IType) -> Result<(), Exception> {
        let old_csr = self.read_csr(args.imm)?;
        self.write_reg(args.rd, old_csr);
        self.write_csr(args.imm, old_csr | args.rs1 as u32)?;
        Ok(())
    }

    fn inst_csrrci(&mut self, args: &IType) -> Result<(), Exception> {
        let old_csr = self.read_csr(args.imm)?;
        self.write_reg(args.rd, old_csr);
        self.write_csr(args.imm, old_csr & !(args.rs1 as u32))?;
        Ok(())
    }

    fn inst_sb(&mut self, args: &SType) {
        let base = self.read_reg(args.rs1);
        let offset = Self::sign_extend(args.imm);
        let addr = (base + offset) as usize;
        // Write least significant byte in rs2.
        let data = self.read_reg(args.rs2) & 0xff;
        self.mem.write_byte(addr, data as u8);
    }

    fn inst_sh(&mut self, args: &SType) {
        let base = self.read_reg(args.rs1);
        let offset = Self::sign_extend(args.imm);
        let addr = (base + offset) as usize;
        // Write least significant 2 byte in rs2.
        let data = self.read_reg(args.rs2) & 0xffff;
        self.mem.write_halfword(addr, data as u16);
    }

    fn inst_sw(&mut self, args: &SType) {
        let base = self.read_reg(args.rs1);
        let offset = Self::sign_extend(args.imm);
        let addr = (base + offset) as usize;
        // Write least significant 4 byte in rs2.
        let data = self.read_reg(args.rs2);
        self.mem.write_word(addr, data);
    }

    // Inner procejure which is common to branch instructions.
    // `offset` is branch instructions' immediate.
    fn branch_inner(&mut self, condition: bool, offset: u16) -> Result<(), Exception> {
        if condition {
            if offset % 4 != 0 {
                // This exception is generated only if the branch condition is true.
                // cf. RISC-V Unprivileged ISA V20191213
                Err(Exception::InstructionAddressMisaligned)
            } else {
                let offset = Self::sign_extend(offset);
                self.pc += offset;
                self.has_jumped = true;
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn inst_beq(&mut self, args: &BType) -> Result<(), Exception> {
        let lv = self.read_reg(args.rs1);
        let rv = self.read_reg(args.rs2);
        self.branch_inner(lv == rv, args.imm)
    }

    fn inst_bne(&mut self, args: &BType) -> Result<(), Exception> {
        let lv = self.read_reg(args.rs1);
        let rv = self.read_reg(args.rs2);
        self.branch_inner(lv != rv, args.imm)
    }

    fn inst_blt(&mut self, args: &BType) -> Result<(), Exception> {
        let lv = self.read_reg(args.rs1) as i32;
        let rv = self.read_reg(args.rs2) as i32;
        self.branch_inner(lv < rv, args.imm)
    }

    fn inst_bge(&mut self, args: &BType) -> Result<(), Exception> {
        let lv = self.read_reg(args.rs1) as i32;
        let rv = self.read_reg(args.rs2) as i32;
        self.branch_inner(lv >= rv, args.imm)
    }

    fn inst_bltu(&mut self, args: &BType) -> Result<(), Exception> {
        let lv = self.read_reg(args.rs1);
        let rv = self.read_reg(args.rs2);
        self.branch_inner(lv < rv, args.imm)
    }

    fn inst_bgeu(&mut self, args: &BType) -> Result<(), Exception> {
        let lv = self.read_reg(args.rs1);
        let rv = self.read_reg(args.rs2);
        self.branch_inner(lv >= rv, args.imm)
    }

    fn inst_auipc(&mut self, args: &UType) {
        let offset = args.imm << 12;
        let new_pc = self.pc + offset;
        self.set_pc(new_pc);
        self.write_reg(args.rd, new_pc);
    }

    fn inst_lui(&mut self, args: &UType) {
        let imm = args.imm << 12;
        self.write_reg(args.rd, imm);
    }

    fn inst_jal(&mut self, args: &JType) -> Result<(), Exception> {
        self.write_reg(args.rd, self.pc + 4);
        let offset = Self::sign_extend_20bit(args.imm);
        let new_pc = (self.pc as i32).wrapping_add(offset) as u32;
        if new_pc % 4 != 0 {
            return Err(Exception::InstructionAddressMisaligned);
        }
        self.set_pc(new_pc);
        self.has_jumped = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csr::address::*;
    use crate::memory::{EmptyMemory, VectorMemory};

    #[test]
    fn calc_rv32i_r_add() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: RType = RType {
            rs1: 1,
            rs2: 2,
            rd: 3,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x3);
        proc.write_reg(2, 0x7);
        proc.inst_add(&args);
        assert_eq!(proc.read_reg(3), 0xa);

        proc.write_reg(1, 0x7fffffff);
        proc.write_reg(2, 0x00007fff);
        proc.inst_add(&args);
        assert_eq!(proc.read_reg(3), 0x80007ffe);
    }

    #[test]
    fn calc_rv32i_r_sub() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: RType = RType {
            rs1: 1,
            rs2: 2,
            rd: 3,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x3);
        proc.write_reg(2, 0x7);
        proc.inst_sub(&args);
        assert_eq!(proc.read_reg(3), 0xfffffffc);

        proc.write_reg(1, 0x7fffffff);
        proc.write_reg(2, 0x00007fff);
        proc.inst_sub(&args);
        assert_eq!(proc.read_reg(3), 0x7fff8000);
    }

    #[test]
    fn calc_rv32i_r_sll() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: RType = RType {
            rs1: 1,
            rs2: 2,
            rd: 3,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x3);
        proc.write_reg(2, 0x7);
        proc.inst_sll(&args);
        assert_eq!(proc.read_reg(3), 0x180);

        proc.write_reg(1, 0xffff1234);
        proc.write_reg(2, 16);
        proc.inst_sll(&args);
        assert_eq!(proc.read_reg(3), 0x12340000);
    }

    #[test]
    fn calc_rv32i_r_slt() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: RType = RType {
            rs1: 1,
            rs2: 2,
            rd: 3,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x3);
        proc.write_reg(2, 0x3);
        proc.inst_slt(&args);
        assert_eq!(proc.read_reg(3), 0x0);

        proc.write_reg(1, 0x3);
        proc.write_reg(2, 0x7);
        proc.inst_slt(&args);
        assert_eq!(proc.read_reg(3), 0x1);

        proc.write_reg(1, 0x7fffffff);
        proc.write_reg(2, 0x00007fff);
        proc.inst_slt(&args);
        assert_eq!(proc.read_reg(3), 0x0);

        proc.write_reg(1, 0xffffffff);
        proc.write_reg(2, 0x00007fff);
        proc.inst_slt(&args);
        assert_eq!(proc.read_reg(3), 0x1);
    }

    #[test]
    fn calc_rv32i_r_sltu() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: RType = RType {
            rs1: 1,
            rs2: 2,
            rd: 3,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x3);
        proc.write_reg(2, 0x3);
        proc.inst_sltu(&args);
        assert_eq!(proc.read_reg(3), 0x0);

        proc.write_reg(1, 0x3);
        proc.write_reg(2, 0x7);
        proc.inst_sltu(&args);
        assert_eq!(proc.read_reg(3), 0x1);

        proc.write_reg(1, 0x7fffffff);
        proc.write_reg(2, 0x00007fff);
        proc.inst_sltu(&args);
        assert_eq!(proc.read_reg(3), 0x0);

        proc.write_reg(1, 0xffffffff);
        proc.write_reg(2, 0x00007fff);
        proc.inst_sltu(&args);
        assert_eq!(proc.read_reg(3), 0x0);
    }

    #[test]
    fn calc_rv32i_r_xor() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: RType = RType {
            rs1: 1,
            rs2: 2,
            rd: 3,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x1234);
        proc.write_reg(2, 0x5678);
        proc.inst_xor(&args);
        assert_eq!(proc.read_reg(3), 0x444c);

        proc.write_reg(1, 0x7fffffff);
        proc.write_reg(2, 0x00007fff);
        proc.inst_xor(&args);
        assert_eq!(proc.read_reg(3), 0x7fff8000);
    }

    #[test]
    fn calc_rv32i_r_srl() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: RType = RType {
            rs1: 1,
            rs2: 2,
            rd: 3,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x1234);
        proc.write_reg(2, 0x4);
        proc.inst_srl(&args);
        assert_eq!(proc.read_reg(3), 0x123);

        proc.write_reg(1, 0x80000000);
        proc.write_reg(2, 0x4);
        proc.inst_srl(&args);
        assert_eq!(proc.read_reg(3), 0x08000000);
    }

    #[test]
    fn calc_rv32i_r_sra() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: RType = RType {
            rs1: 1,
            rs2: 2,
            rd: 3,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x1234);
        proc.write_reg(2, 0x4);
        proc.inst_sra(&args);
        assert_eq!(proc.read_reg(3), 0x123);

        proc.write_reg(1, 0x80000000);
        proc.write_reg(2, 0x4);
        proc.inst_sra(&args);
        assert_eq!(proc.read_reg(3), 0xf8000000);
    }

    #[test]
    fn calc_rv32i_r_and() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: RType = RType {
            rs1: 1,
            rs2: 2,
            rd: 3,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x1234);
        proc.write_reg(2, 0x5678);
        proc.inst_and(&args);
        assert_eq!(proc.read_reg(3), 0x1230);

        proc.write_reg(1, 0x7fffffff);
        proc.write_reg(2, 0x00007fff);
        proc.inst_and(&args);
        assert_eq!(proc.read_reg(3), 0x00007fff);
    }

    #[test]
    fn calc_rv32i_r_or() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: RType = RType {
            rs1: 1,
            rs2: 2,
            rd: 3,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x1234);
        proc.write_reg(2, 0x5678);
        proc.inst_or(&args);
        assert_eq!(proc.read_reg(3), 0x567c);

        proc.write_reg(1, 0x7fff8000);
        proc.write_reg(2, 0x00007fff);
        proc.inst_or(&args);
        assert_eq!(proc.read_reg(3), 0x7fffffff);
    }

    #[test]
    fn calc_rv32i_i_jalr() -> Result<(), Exception> {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: IType = IType {
            rs1: 1,
            rd: 2,
            imm: 0x111,
        };

        let mut proc = Processor::new(memory);
        proc.set_pc(0x1234);

        proc.write_reg(1, 0x567);
        proc.inst_jalr(&args)?;
        assert_eq!(proc.read_reg(2), 0x1238);
        assert_eq!(proc.pc, 0x678);

        proc.pc = 0x1234;
        proc.write_reg(1, 0x543);
        proc.inst_jalr(&args)?;
        assert_eq!(proc.read_reg(2), 0x1238);
        assert_eq!(proc.pc, 0x654);
        Ok(())
    }

    #[test]
    fn calc_rv32i_i_jalr_invalid_address() -> Result<(), Exception> {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: IType = IType {
            rs1: 1,
            rd: 2,
            imm: 0x110,
        };

        let mut proc = Processor::new(memory);

        proc.pc = 0x1234;
        proc.write_reg(1, 0x567);
        // x1 == 0x677, which is not aligned to a 4byte boundary.
        assert_eq!(
            proc.inst_jalr(&args),
            Err(Exception::InstructionAddressMisaligned)
        );
        Ok(())
    }

    #[test]
    fn calc_rv32i_i_addi() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: IType = IType {
            rs1: 1,
            rd: 2,
            imm: 0x123,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x567);
        proc.inst_addi(&args);
        assert_eq!(proc.read_reg(2), 0x68a);
    }

    #[test]
    fn calc_rv32i_i_slli() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: IType = IType {
            rs1: 1,
            rd: 2,
            imm: 0x3,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x5678);
        proc.inst_slli(&args);
        assert_eq!(proc.read_reg(2), 0x2b3c0);
    }

    #[test]
    fn calc_rv32i_i_slti() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: IType = IType {
            rs1: 1,
            rd: 2,
            imm: 0x123,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x567);
        proc.inst_slti(&args);
        assert_eq!(proc.read_reg(2), 0x0);

        proc.write_reg(1, 0x0);
        proc.inst_slti(&args);
        assert_eq!(proc.read_reg(2), 0x1);

        proc.write_reg(1, 0xffffffff);
        proc.inst_slti(&args);
        assert_eq!(proc.read_reg(2), 0x1);
    }

    #[test]
    fn calc_rv32i_i_sltiu() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: IType = IType {
            rs1: 1,
            rd: 2,
            imm: 0x123,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x5678);
        proc.inst_sltiu(&args);
        assert_eq!(proc.read_reg(2), 0x0);

        proc.write_reg(1, 0x0);
        proc.inst_sltiu(&args);
        assert_eq!(proc.read_reg(2), 0x1);

        proc.write_reg(1, 0xffffffff);
        proc.inst_sltiu(&args);
        assert_eq!(proc.read_reg(2), 0x0);
    }

    #[test]
    fn calc_rv32i_i_xori() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: IType = IType {
            rs1: 1,
            rd: 2,
            imm: 0x123,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x5678);
        proc.inst_xori(&args);
        assert_eq!(proc.read_reg(2), 0x575b);
    }

    #[test]
    fn calc_rv32i_i_srli() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: IType = IType {
            rs1: 1,
            rd: 2,
            imm: 0x3,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x5678);
        proc.inst_srli(&args);
        assert_eq!(proc.read_reg(2), 0xacf);

        proc.write_reg(1, 0x80000000);
        proc.inst_srli(&args);
        assert_eq!(proc.read_reg(2), 0x10000000);
    }

    #[test]
    fn calc_rv32i_i_srai() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: IType = IType {
            rs1: 1,
            rd: 2,
            imm: 0x3,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x5678);
        proc.inst_srai(&args);
        assert_eq!(proc.read_reg(2), 0xacf);

        proc.write_reg(1, 0x80000000);
        proc.inst_srai(&args);
        assert_eq!(proc.read_reg(2), 0xf0000000);
    }

    #[test]
    fn calc_rv32i_i_ori() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: IType = IType {
            rs1: 1,
            rd: 2,
            imm: 0x123,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x5678);
        proc.inst_ori(&args);
        assert_eq!(proc.read_reg(2), 0x577b);
    }

    #[test]
    fn calc_rv32i_i_andi() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args: IType = IType {
            rs1: 1,
            rd: 2,
            imm: 0x123,
        };

        let mut proc = Processor::new(memory);

        proc.write_reg(1, 0x5678);
        proc.inst_andi(&args);
        assert_eq!(proc.read_reg(2), 0x020);
    }

    #[test]
    fn calc_rv32i_i_load() {
        let memory = vec![0x0, 0x0, 0x0, 0x0, 0x80, 0x80, 0x08, 0x08];
        let memory: Box<dyn Memory> = Box::new(VectorMemory::from(memory));
        let args: IType = IType {
            rs1: 1,
            rd: 2,
            imm: 0x0,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(1, 4);

        proc.inst_lb(&args);
        assert_eq!(proc.read_reg(2), 0xffffff80);

        proc.inst_lh(&args);
        assert_eq!(proc.read_reg(2), 0xffff8080);

        proc.inst_lw(&args);
        assert_eq!(proc.read_reg(2), 0x08088080);

        proc.inst_lbu(&args);
        assert_eq!(proc.read_reg(2), 0x80);

        proc.inst_lhu(&args);
        assert_eq!(proc.read_reg(2), 0x8080);

        let args: IType = IType {
            rs1: 1,
            rd: 2,
            imm: 0x4,
        };

        proc.write_reg(1, 0);

        proc.inst_lb(&args);
        assert_eq!(proc.read_reg(2), 0xffffff80);

        proc.inst_lh(&args);
        assert_eq!(proc.read_reg(2), 0xffff8080);

        proc.inst_lw(&args);
        assert_eq!(proc.read_reg(2), 0x08088080);

        proc.inst_lbu(&args);
        assert_eq!(proc.read_reg(2), 0x80);

        proc.inst_lhu(&args);
        assert_eq!(proc.read_reg(2), 0x8080);
    }

    #[test]
    fn calc_rv32i_i_csrrw() -> Result<(), Exception> {
        let memory = vec![0; 8];
        let memory: Box<dyn Memory> = Box::new(VectorMemory::from(memory));
        let args = IType {
            rd: 1,
            rs1: 2,
            imm: UTVEC as u16,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(args.rs1, 0x2);
        proc.write_csr(args.imm, 0x1)?;
        proc.inst_csrrw(&args)?;
        assert_eq!(proc.read_reg(args.rd), 0x1); // rd = CSR[args.imm]
        assert_eq!(proc.read_csr(args.imm)?, 0x2); // CSR[args.imm] = rs1
        Ok(())
    }

    #[test]
    fn calc_rv32i_i_csrrs() -> Result<(), Exception> {
        let memory = vec![0; 8];
        let memory: Box<dyn Memory> = Box::new(VectorMemory::from(memory));
        let args = IType {
            rd: 1,
            rs1: 2,
            imm: UTVEC as u16,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(args.rs1, 0x2);
        proc.write_csr(args.imm, 0x1)?;
        proc.inst_csrrs(&args)?;
        assert_eq!(proc.read_reg(args.rd), 0x1); // rd = CSR[args.imm]
        assert_eq!(proc.read_csr(args.imm)?, 0x3); // CSR[args.imm] |= rs1
        Ok(())
    }

    #[test]
    fn calc_rv32i_i_csrrc() -> Result<(), Exception> {
        let memory = vec![0; 8];
        let memory: Box<dyn Memory> = Box::new(VectorMemory::from(memory));
        let args = IType {
            rd: 1,
            rs1: 2,
            imm: UTVEC as u16,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(args.rs1, 0x2);
        proc.write_csr(args.imm, 0x4)?;
        proc.inst_csrrc(&args)?;
        assert_eq!(proc.read_reg(args.rd), 0x4); // rd = CSR[args.imm]
        assert_eq!(proc.read_csr(args.imm)?, 0x4); // CSR[args.imm] &= !rs1
        Ok(())
    }

    #[test]
    fn calc_rv32i_i_csrrwi() -> Result<(), Exception> {
        let memory = vec![0; 8];
        let memory: Box<dyn Memory> = Box::new(VectorMemory::from(memory));
        let args = IType {
            rd: 1,
            rs1: 2,
            imm: UTVEC as u16,
        };

        let mut proc = Processor::new(memory);
        proc.write_csr(args.imm, 0x1)?;
        proc.inst_csrrwi(&args)?;
        assert_eq!(proc.read_reg(args.rd), 0x1); // rd = CSR[args.imm]
        assert_eq!(proc.read_csr(args.imm)?, 0x2); // CSR[args.imm] = rs1
        Ok(())
    }

    #[test]
    fn calc_rv32i_i_csrrsi() -> Result<(), Exception> {
        let memory = vec![0; 8];
        let memory: Box<dyn Memory> = Box::new(VectorMemory::from(memory));
        let args = IType {
            rd: 1,
            rs1: 2,
            imm: UTVEC as u16,
        };

        let mut proc = Processor::new(memory);
        proc.write_csr(args.imm, 0x1)?;
        proc.inst_csrrsi(&args)?;
        assert_eq!(proc.read_reg(args.rd), 0x1); // rd = CSR[args.imm]
        assert_eq!(proc.read_csr(args.imm)?, 0x3); // CSR[args.imm] &= !rs1
        Ok(())
    }

    #[test]
    fn calc_rv32i_i_csrrci() -> Result<(), Exception> {
        let memory = vec![0; 8];
        let memory: Box<dyn Memory> = Box::new(VectorMemory::from(memory));
        let args = IType {
            rd: 1,
            rs1: 2,
            imm: UTVEC as u16,
        };

        let mut proc = Processor::new(memory);
        proc.write_csr(args.imm, 0x4)?;
        proc.inst_csrrci(&args)?;
        assert_eq!(proc.read_reg(args.rd), 0x4); // rd = CSR[args.imm]
        assert_eq!(proc.read_csr(args.imm)?, 0x4); // CSR[args.imm] |= rs1
        Ok(())
    }

    #[test]
    fn calc_rv32i_s_sb() {
        let memory = vec![0; 8];
        let memory: Box<dyn Memory> = Box::new(VectorMemory::from(memory));
        let args = SType {
            rs1: 1,
            rs2: 2,
            imm: 0x2,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(1, 0x2);
        proc.write_reg(2, 0x180);
        proc.inst_sb(&args);
        assert_eq!(proc.mem.read_byte(4), 0x80);
    }

    #[test]
    fn calc_rv32i_s_sh() {
        let memory = vec![0; 8];
        let memory: Box<dyn Memory> = Box::new(VectorMemory::from(memory));
        let args = SType {
            rs1: 1,
            rs2: 2,
            imm: 0x2,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(1, 0x2);
        proc.write_reg(2, 0x18080);
        proc.inst_sh(&args);
        assert_eq!(proc.mem.read_halfword(4), 0x8080);
    }

    #[test]
    fn calc_rv32i_s_sw() {
        let memory = vec![0; 8];
        let memory: Box<dyn Memory> = Box::new(VectorMemory::from(memory));
        let args = SType {
            rs1: 1,
            rs2: 2,
            imm: 0x2,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(1, 0x2);
        proc.write_reg(2, 0x80808080);
        proc.inst_sw(&args);
        assert_eq!(proc.mem.read_word(4), 0x80808080);
    }

    #[test]
    fn calc_rv32i_b_beq() -> Result<(), Exception> {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args = BType {
            rs1: 1,
            rs2: 2,
            imm: 0x80,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(1, 42);
        proc.write_reg(2, 42);
        proc.inst_beq(&args)?;
        assert_eq!(proc.pc, 0x80);
        Ok(())
    }

    // Test for invalid address in branch instruction is enough for this case because a processing the
    // exception is abstracted in `Processor::branch_inner()`.
    #[test]
    fn calc_rv32i_b_beq_invalid_address() -> Result<(), Exception> {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args = BType {
            rs1: 1,
            rs2: 2,
            imm: 0x81,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(1, 42);
        proc.write_reg(2, 42);
        assert_eq!(
            proc.inst_beq(&args),
            Err(Exception::InstructionAddressMisaligned)
        );
        Ok(())
    }

    #[test]
    fn calc_rv32i_b_bne() -> Result<(), Exception> {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args = BType {
            rs1: 1,
            rs2: 2,
            imm: 0x80,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(1, 42);
        proc.write_reg(2, 0);
        proc.inst_bne(&args)?;
        assert_eq!(proc.pc, 0x80);
        Ok(())
    }

    #[test]
    fn calc_rv32i_b_blt() -> Result<(), Exception> {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args = BType {
            rs1: 1,
            rs2: 2,
            imm: 0x80,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(1, 0xffffff80);
        proc.write_reg(2, 0);
        // Compare register values as signed value.
        proc.inst_blt(&args)?;
        assert_eq!(proc.pc, 0x80);
        Ok(())
    }

    #[test]
    fn calc_rv32i_b_bgt() -> Result<(), Exception> {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args = BType {
            rs1: 1,
            rs2: 2,
            imm: 0x80,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(1, 0);
        proc.write_reg(2, 0xffffff80);
        // Compare register values as signed value.
        proc.inst_bge(&args)?;
        assert_eq!(proc.pc, 0x80);

        proc.write_reg(1, 0xffffff80);
        proc.write_reg(2, 0xffffff80);
        // Compare register values as signed value.
        proc.inst_bge(&args)?;
        assert_eq!(proc.pc, 0x100);
        Ok(())
    }

    #[test]
    fn calc_rv32i_b_bltu() -> Result<(), Exception> {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args = BType {
            rs1: 1,
            rs2: 2,
            imm: 0x80,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(1, 0);
        proc.write_reg(2, 0xffffff80);
        // Compare register values as unsigned value.
        proc.inst_bltu(&args)?;
        assert_eq!(proc.pc, 0x80);
        Ok(())
    }

    #[test]
    fn calc_rv32i_b_bgtu() -> Result<(), Exception> {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args = BType {
            rs1: 1,
            rs2: 2,
            imm: 0x80,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(1, 0xffffff80);
        proc.write_reg(2, 0);
        // Compare register values as unsigned value.
        proc.inst_bgeu(&args)?;
        assert_eq!(proc.pc, 0x80);

        proc.write_reg(1, 0xffffff80);
        proc.write_reg(2, 0xffffff80);
        // Compare register values as signed value.
        proc.inst_bgeu(&args)?;
        assert_eq!(proc.pc, 0x100);
        Ok(())
    }

    #[test]
    fn calc_rv32i_u_lui() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args = UType {
            rd: 1,
            imm: 0xfffff,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(1, 0x0);
        proc.inst_lui(&args);
        assert_eq!(proc.read_reg(args.rd), 0xfffff000);
    }

    #[test]
    fn calc_rv32i_u_auipc() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args = UType {
            rd: 1,
            imm: 0xfffff,
        };

        let mut proc = Processor::new(memory);
        proc.write_reg(1, 0x0);
        // If pc is 0, cannot detect not adding `imm` to current pc.
        proc.set_pc(0x4);
        proc.inst_auipc(&args);
        assert_eq!(proc.read_reg(args.rd), 0xfffff004);
        assert_eq!(proc.pc, 0xfffff004);
    }

    #[test]
    fn calc_rv32i_j_jal() -> Result<(), Exception> {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args = JType { rd: 1, imm: 0x80 };

        let mut proc = Processor::new(memory);
        proc.write_reg(1, 0x0);
        proc.set_pc(0x4);
        proc.inst_jal(&args)?;
        assert_eq!(proc.read_reg(args.rd), 0x8);
        assert_eq!(proc.pc, 0x84);

        let args = JType {
            rd: 1,
            imm: 0xfffffffc, // -4
        };
        proc.inst_jal(&args)?;
        assert_eq!(proc.read_reg(args.rd), 0x88);
        assert_eq!(proc.pc, 0x80);
        Ok(())
    }

    #[test]
    fn calc_rv32i_j_jal_invalid_address() -> Result<(), Exception> {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory);
        let args = JType { rd: 1, imm: 0x82 };

        let mut proc = Processor::new(memory);
        proc.write_reg(1, 0x0);
        assert_eq!(
            proc.inst_jal(&args),
            Err(Exception::InstructionAddressMisaligned)
        );
        Ok(())
    }
}
