use bit_field::BitField;
use std::ops::Range;

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Add(RType),
    Sub(RType),
    Sll(RType),
    Slt(RType),
    Sltu(RType),
    Xor(RType),
    Srl(RType),
    Sra(RType),
    Or(RType),
    And(RType),
}

#[derive(Debug, PartialEq, Eq)]
pub struct RType {
    rd: u8,
    rs1: u8,
    rs2: u8,
}

impl RType {
    fn new(instruction: u32) -> Self {
        Self {
            rd: instruction.get_bits(7..12) as u8,
            rs1: instruction.get_bits(15..20) as u8,
            rs2: instruction.get_bits(20..25) as u8,
        }
    }
}

const OPCODE_RANGE: Range<usize> = 0..7;
const FUNCT3_RANGE: Range<usize> = 25..32;
const FUNCT7_RANGE: Range<usize> = 12..15;

pub fn decode(instruction: u32) -> Instruction {
    match instruction.get_bits(OPCODE_RANGE) {
        // R Type
        0b0110011 => match instruction.get_bits(FUNCT7_RANGE) {
            0b000 => match instruction.get_bits(FUNCT3_RANGE) {
                0b0000000 => Instruction::Add(RType::new(instruction)),
                0b0100000 => Instruction::Sub(RType::new(instruction)),
                _ => panic!("Invalid instruction"),
            },
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_rv32i_r() {
        // add x1, x9, x5
        let add = 0b0000000_00101_01001_000_00001_0110011;
        assert_eq!(
            Instruction::Add(RType {
                rd: 1,
                rs1: 9,
                rs2: 5,
            }),
            decode(add)
        );

        // sub x2, x6, x21
        let sub = 0b0100000_10101_00110_000_00010_0110011;
        assert_eq!(
            Instruction::Sub(RType {
                rd: 2,
                rs1: 6,
                rs2: 21,
            }),
            decode(sub)
        );
    }
}
