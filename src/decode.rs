use bit_field::BitField;
use std::ops::Range;

const OPCODE_RANGE: Range<usize> = 0..7;
const RD_RANGE: Range<usize> = 7..12;
const RS1_RANGE: Range<usize> = 15..20;
const RS2_RANGE: Range<usize> = 20..25;
const FUNCT3_RANGE: Range<usize> = 12..15;
const FUNCT7_RANGE: Range<usize> = 25..32;

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
            rd: instruction.get_bits(RD_RANGE) as u8,
            rs1: instruction.get_bits(RS1_RANGE) as u8,
            rs2: instruction.get_bits(RS2_RANGE) as u8,
        }
    }
}

pub fn decode(instruction: u32) -> Instruction {
    match instruction.get_bits(OPCODE_RANGE) {
        // R Type
        0b0110011 => match instruction.get_bits(FUNCT3_RANGE) {
            0b000 => match instruction.get_bits(FUNCT7_RANGE) {
                0b0000000 => Instruction::Add(RType::new(instruction)),
                0b0100000 => Instruction::Sub(RType::new(instruction)),
                _ => panic!("Invalid instruction"),
            },
            0b001 => Instruction::Sll(RType::new(instruction)),
            0b010 => Instruction::Slt(RType::new(instruction)),
            0b011 => Instruction::Sltu(RType::new(instruction)),
            0b100 => Instruction::Xor(RType::new(instruction)),
            0b101 => match instruction.get_bits(FUNCT7_RANGE) {
                0b0000000 => Instruction::Srl(RType::new(instruction)),
                0b0100000 => Instruction::Sra(RType::new(instruction)),
                _ => panic!("Invalid instruction"),
            },
            0b110 => Instruction::Or(RType::new(instruction)),
            0b111 => Instruction::And(RType::new(instruction)),
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
        assert_eq!(
            Instruction::Add(RType {
                rd: 1,
                rs1: 9,
                rs2: 5,
            }),
            decode(0b0000000_00101_01001_000_00001_0110011)
        );

        // sub x2, x6, x21
        assert_eq!(
            Instruction::Sub(RType {
                rd: 2,
                rs1: 6,
                rs2: 21,
            }),
            decode(0b0100000_10101_00110_000_00010_0110011)
        );

        // sll x3, x4, x24
        assert_eq!(
            Instruction::Sll(RType {
                rd: 3,
                rs1: 4,
                rs2: 24,
            }),
            decode(0b0000000_11000_00100_001_00011_0110011)
        );

        // slt x4, x19, x31
        assert_eq!(
            Instruction::Slt(RType {
                rd: 4,
                rs1: 19,
                rs2: 31,
            }),
            decode(0b0000000_11111_10011_010_00100_0110011)
        );

        // sltu x5, x12, x11
        assert_eq!(
            Instruction::Sltu(RType {
                rd: 5,
                rs1: 12,
                rs2: 11,
            }),
            decode(0b0000000_01011_01100_011_00101_0110011)
        );

        // xor x6, x17, x25
        assert_eq!(
            Instruction::Xor(RType {
                rd: 6,
                rs1: 17,
                rs2: 25,
            }),
            decode(0b0000000_11001_10001_100_00110_0110011)
        );

        // srl x7, x27, x15
        assert_eq!(
            Instruction::Srl(RType {
                rd: 7,
                rs1: 27,
                rs2: 15,
            }),
            decode(0b0000000_01111_11011_101_00111_0110011)
        );

        // sra x8, x13, x28
        assert_eq!(
            Instruction::Sra(RType {
                rd: 8,
                rs1: 13,
                rs2: 28,
            }),
            decode(0b0100000_11100_01101_101_01000_0110011)
        );

        // or x9, x30, x25
        assert_eq!(
            Instruction::Or(RType {
                rd: 9,
                rs1: 30,
                rs2: 25,
            }),
            decode(0b0000000_11001_11110_110_01001_0110011)
        );

        // xor x10, x17, x0
        assert_eq!(
            Instruction::And(RType {
                rd: 10,
                rs1: 17,
                rs2: 0,
            }),
            decode(0b0000000_00000_10001_111_01010_0110011)
        );
    }
}
