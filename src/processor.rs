use crate::memory::Memory;

use crate::decode::{decode, Instruction, RType};

pub struct Processor {
    pub regs: [u32; 32],
    pub pc: u32,
    pub mem: Box<dyn Memory>,
}

impl Processor {
    fn new(memory: Box<dyn Memory>) -> Self {
        Self {
            regs: [0; 32],
            pc: 0,
            mem: memory,
        }
    }

    fn read_reg(&self, idx: usize) -> u32 {
        if idx == 0 {
            0
        }
        else {
            self.regs[idx]
        }
    }
    
    fn write_reg(&mut self, idx: usize, val: u32) {
        if idx != 0 {
            self.regs[idx] = val;
        }
    }

    fn tick(&mut self) {
        let raw_inst = self.mem.read_inst(self.pc as usize);
        match decode(raw_inst) {
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
            _ => panic!("unimplemented"),
        }
    }
}

impl Processor {
    fn inst_add(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1 as usize);
        let rv = self.read_reg(args.rs2 as usize);
        let v = lv.wrapping_add(rv); 
        self.write_reg(args.rd as usize, v);
    }

    fn inst_sub(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1 as usize);
        let rv = self.read_reg(args.rs2 as usize);
        let v = lv.wrapping_sub(rv); 
        self.write_reg(args.rd as usize, v);
    }

    fn inst_sll(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1 as usize);
        let rv = self.read_reg(args.rs2 as usize);
        let v = lv << rv; 
        self.write_reg(args.rd as usize, v);
    }

    fn inst_slt(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1 as usize) as i32;
        let rv = self.read_reg(args.rs2 as usize) as i32;
        let v = (lv < rv) as u32; 
        self.write_reg(args.rd as usize, v);
    }

    fn inst_sltu(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1 as usize);
        let rv = self.read_reg(args.rs2 as usize);
        let v = (lv < rv) as u32; 
        self.write_reg(args.rd as usize, v);
    }

    fn inst_xor(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1 as usize);
        let rv = self.read_reg(args.rs2 as usize);
        let v = lv ^ rv; 
        self.write_reg(args.rd as usize, v);
    }

    fn inst_srl(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1 as usize);
        let rv = self.read_reg(args.rs2 as usize);
        let v = lv >> rv; 
        self.write_reg(args.rd as usize, v);
    }

    fn inst_sra(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1 as usize) as i32;
        let rv = self.read_reg(args.rs2 as usize);
        let v = (lv >> rv) as u32; 
        self.write_reg(args.rd as usize, v);
    }

    fn inst_or(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1 as usize);
        let rv = self.read_reg(args.rs2 as usize);
        let v = lv | rv;
        self.write_reg(args.rd as usize, v);
    }

    fn inst_and(&mut self, args: &RType) {
        let lv = self.read_reg(args.rs1 as usize);
        let rv = self.read_reg(args.rs2 as usize);
        let v = lv & rv;
        self.write_reg(args.rd as usize, v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::decode::RType;
    use crate::memory::EmptyMemory;

    #[test]
    fn calc_rv32i_r_add() {
        let memory: Box<dyn Memory> = Box::new(EmptyMemory::new());
        let args: RType = RType { rs1: 1, rs2: 2, rd: 3 };

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
        let memory: Box<dyn Memory> = Box::new(EmptyMemory::new());
        let args: RType = RType { rs1: 1, rs2: 2, rd: 3 };

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
        let memory: Box<dyn Memory> = Box::new(EmptyMemory::new());
        let args: RType = RType { rs1: 1, rs2: 2, rd: 3 };

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
        let memory: Box<dyn Memory> = Box::new(EmptyMemory::new());
        let args: RType = RType { rs1: 1, rs2: 2, rd: 3 };

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
        let memory: Box<dyn Memory> = Box::new(EmptyMemory::new());
        let args: RType = RType { rs1: 1, rs2: 2, rd: 3 };

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
        let memory: Box<dyn Memory> = Box::new(EmptyMemory::new());
        let args: RType = RType { rs1: 1, rs2: 2, rd: 3 };

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
        let memory: Box<dyn Memory> = Box::new(EmptyMemory::new());
        let args: RType = RType { rs1: 1, rs2: 2, rd: 3 };

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
        let memory: Box<dyn Memory> = Box::new(EmptyMemory::new());
        let args: RType = RType { rs1: 1, rs2: 2, rd: 3 };

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
        let memory: Box<dyn Memory> = Box::new(EmptyMemory::new());
        let args: RType = RType { rs1: 1, rs2: 2, rd: 3 };

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
        let memory: Box<dyn Memory> = Box::new(EmptyMemory::new());
        let args: RType = RType { rs1: 1, rs2: 2, rd: 3 };

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
}
