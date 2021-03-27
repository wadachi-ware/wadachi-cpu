use crate::exception::Exception;
use bit_field::BitField;
use std::ops::Range;

const OPCODE_RANGE: Range<usize> = 0..7;
const RD_RANGE: Range<usize> = 7..12;
const RS1_RANGE: Range<usize> = 15..20;
const RS2_RANGE: Range<usize> = 20..25;
const FUNCT3_RANGE: Range<usize> = 12..15;
const FUNCT7_RANGE: Range<usize> = 25..32;
const IMM_RANGE: Range<usize> = 20..32;
const UPPER_IMM_RANGE: Range<usize> = 12..32;

/// Enumerates instructions.
/// Each entry have a struct holding parameters such as register index.
#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    // R-Type
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

    // I-Type
    Jalr(IType),
    Addi(IType),
    Slli(IType),
    Slti(IType),
    Sltiu(IType),
    Xori(IType),
    Srli(IType),
    Srai(IType),
    Ori(IType),
    Andi(IType),
    Lb(IType),
    Lh(IType),
    Lw(IType),
    Lbu(IType),
    Lhu(IType),
    Csrrw(IType),
    Csrrs(IType),
    Csrrc(IType),
    Csrrwi(IType),
    Csrrsi(IType),
    Csrrci(IType),

    // S-Type
    Sb(SType),
    Sh(SType),
    Sw(SType),

    // B-Type*
    Beq(BType),
    Bne(BType),
    Blt(BType),
    Bge(BType),
    Bltu(BType),
    Bgeu(BType),

    // J-Type
    Jal(JType),

    // U-Type
    Lui(UType),
    Auipc(UType),
}

/// Parameters common to R-Type instructions.
/// This is the same for structs below.
#[derive(Debug, PartialEq, Eq)]
pub struct RType {
    pub rd: usize,
    pub rs1: usize,
    pub rs2: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct IType {
    pub rd: usize,
    pub rs1: usize,
    pub imm: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SType {
    pub rs1: usize,
    pub rs2: usize,
    pub imm: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BType {
    pub rs1: usize,
    pub rs2: usize,
    pub imm: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub struct UType {
    pub rd: usize,
    pub imm: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct JType {
    pub rd: usize,
    pub imm: u32,
}

impl RType {
    fn new(instruction: u32) -> Self {
        Self {
            rd: instruction.get_bits(RD_RANGE) as usize,
            rs1: instruction.get_bits(RS1_RANGE) as usize,
            rs2: instruction.get_bits(RS2_RANGE) as usize,
        }
    }
}

impl IType {
    fn new(instruction: u32) -> Self {
        Self {
            rd: instruction.get_bits(RD_RANGE) as usize,
            rs1: instruction.get_bits(RS1_RANGE) as usize,
            imm: instruction.get_bits(IMM_RANGE) as u16,
        }
    }
}

impl SType {
    fn new(instruction: u32) -> Self {
        let imm = instruction.get_bits(7..12) + (instruction.get_bits(25..32) << 5);
        Self {
            rs1: instruction.get_bits(RS1_RANGE) as usize,
            rs2: instruction.get_bits(RS2_RANGE) as usize,
            imm: imm as u16,
        }
    }
}

impl BType {
    fn new(instruction: u32) -> Self {
        let imm = (instruction.get_bits(8..12)
            + (instruction.get_bits(25..31) << 4)
            + (instruction.get_bits(7..8) << 10)
            + (instruction.get_bits(31..32) << 11))
            << 1;
        Self {
            rs1: instruction.get_bits(RS1_RANGE) as usize,
            rs2: instruction.get_bits(RS2_RANGE) as usize,
            imm: imm as u16,
        }
    }
}

impl UType {
    fn new(instruction: u32) -> Self {
        let imm = instruction.get_bits(UPPER_IMM_RANGE) << 12;
        Self {
            rd: instruction.get_bits(RD_RANGE) as usize,
            imm,
        }
    }
}

impl JType {
    fn new(instruction: u32) -> Self {
        let imm = (instruction.get_bits(21..31)
            + (instruction.get_bits(20..21) << 10)
            + (instruction.get_bits(12..20) << 11)
            + (instruction.get_bits(31..32) << 19))
            << 1;
        Self {
            rd: instruction.get_bits(RD_RANGE) as usize,
            imm,
        }
    }
}

/// Decode an instruction.
pub fn decode(instruction: u32) -> Result<Instruction, Exception> {
    let decoded = match instruction.get_bits(OPCODE_RANGE) {
        // R-Type
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
            _ => return Err(Exception::IllegalInstruction),
        },

        // I Type
        0b1100111 => {
            let decoded = IType::new(instruction);
            if decoded.imm % 4 != 0 {
                return Err(Exception::InstructionAddressMisaligned);
            }
            Instruction::Jalr(decoded)
        }
        0b0010011 => match instruction.get_bits(FUNCT3_RANGE) {
            0b000 => Instruction::Addi(IType::new(instruction)),
            0b001 => Instruction::Slli(IType::new(instruction)),
            0b010 => Instruction::Slti(IType::new(instruction)),
            0b011 => Instruction::Sltiu(IType::new(instruction)),
            0b100 => Instruction::Xori(IType::new(instruction)),
            0b101 => match instruction.get_bits(FUNCT7_RANGE) {
                0b0000000 => Instruction::Srli(IType::new(instruction)),
                0b0100000 => Instruction::Srai(IType::new(instruction)),
                _ => return Err(Exception::IllegalInstruction),
            },
            0b110 => Instruction::Ori(IType::new(instruction)),
            0b111 => Instruction::Andi(IType::new(instruction)),
            _ => return Err(Exception::IllegalInstruction),
        },
        0b0000011 => match instruction.get_bits(FUNCT3_RANGE) {
            0b000 => Instruction::Lb(IType::new(instruction)),
            0b001 => Instruction::Lh(IType::new(instruction)),
            0b010 => Instruction::Lw(IType::new(instruction)),
            0b100 => Instruction::Lbu(IType::new(instruction)),
            0b101 => Instruction::Lhu(IType::new(instruction)),
            _ => return Err(Exception::IllegalInstruction),
        },
        0b1110011 => match instruction.get_bits(FUNCT3_RANGE) {
            0b001 => Instruction::Csrrw(IType::new(instruction)),
            0b010 => Instruction::Csrrs(IType::new(instruction)),
            0b011 => Instruction::Csrrc(IType::new(instruction)),
            0b101 => Instruction::Csrrwi(IType::new(instruction)),
            0b110 => Instruction::Csrrsi(IType::new(instruction)),
            0b111 => Instruction::Csrrci(IType::new(instruction)),
            _ => return Err(Exception::IllegalInstruction),
        },

        // S-Type
        0b0100011 => match instruction.get_bits(FUNCT3_RANGE) {
            0b000 => Instruction::Sb(SType::new(instruction)),
            0b001 => Instruction::Sh(SType::new(instruction)),
            0b010 => Instruction::Sw(SType::new(instruction)),
            _ => return Err(Exception::IllegalInstruction),
        },

        // B-Type
        0b1100011 => match instruction.get_bits(FUNCT3_RANGE) {
            0b000 => Instruction::Beq(BType::new(instruction)),
            0b001 => Instruction::Bne(BType::new(instruction)),
            0b100 => Instruction::Blt(BType::new(instruction)),
            0b101 => Instruction::Bge(BType::new(instruction)),
            0b110 => Instruction::Bltu(BType::new(instruction)),
            0b111 => Instruction::Bgeu(BType::new(instruction)),
            _ => return Err(Exception::IllegalInstruction),
        },

        // J-Type
        0b1101111 => Instruction::Jal(JType::new(instruction)),

        // U-Type
        0b0110111 => Instruction::Lui(UType::new(instruction)),
        0b0010111 => Instruction::Auipc(UType::new(instruction)),
        _ => return Err(Exception::IllegalInstruction),
    };
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_rv32i_r() -> Result<(), Exception> {
        // add x1, x9, x5
        assert_eq!(
            Instruction::Add(RType {
                rd: 1,
                rs1: 9,
                rs2: 5,
            }),
            decode(0b0000000_00101_01001_000_00001_0110011)?
        );

        // sub x2, x6, x21
        assert_eq!(
            Instruction::Sub(RType {
                rd: 2,
                rs1: 6,
                rs2: 21,
            }),
            decode(0b0100000_10101_00110_000_00010_0110011)?
        );

        // sll x3, x4, x24
        assert_eq!(
            Instruction::Sll(RType {
                rd: 3,
                rs1: 4,
                rs2: 24,
            }),
            decode(0b0000000_11000_00100_001_00011_0110011)?
        );

        // slt x4, x19, x31
        assert_eq!(
            Instruction::Slt(RType {
                rd: 4,
                rs1: 19,
                rs2: 31,
            }),
            decode(0b0000000_11111_10011_010_00100_0110011)?
        );

        // sltu x5, x12, x11
        assert_eq!(
            Instruction::Sltu(RType {
                rd: 5,
                rs1: 12,
                rs2: 11,
            }),
            decode(0b0000000_01011_01100_011_00101_0110011)?
        );

        // xor x6, x17, x25
        assert_eq!(
            Instruction::Xor(RType {
                rd: 6,
                rs1: 17,
                rs2: 25,
            }),
            decode(0b0000000_11001_10001_100_00110_0110011)?
        );

        // srl x7, x27, x15
        assert_eq!(
            Instruction::Srl(RType {
                rd: 7,
                rs1: 27,
                rs2: 15,
            }),
            decode(0b0000000_01111_11011_101_00111_0110011)?
        );

        // sra x8, x13, x28
        assert_eq!(
            Instruction::Sra(RType {
                rd: 8,
                rs1: 13,
                rs2: 28,
            }),
            decode(0b0100000_11100_01101_101_01000_0110011)?
        );

        // or x9, x30, x25
        assert_eq!(
            Instruction::Or(RType {
                rd: 9,
                rs1: 30,
                rs2: 25,
            }),
            decode(0b0000000_11001_11110_110_01001_0110011)?
        );

        // and x10, x17, x0
        assert_eq!(
            Instruction::And(RType {
                rd: 10,
                rs1: 17,
                rs2: 0,
            }),
            decode(0b0000000_00000_10001_111_01010_0110011)?
        );
        Ok(())
    }

    #[test]
    fn decode_rv32i_i() -> Result<(), Exception> {
        // jalr x1, x9, 64
        assert_eq!(
            Instruction::Jalr(IType {
                rd: 1,
                rs1: 9,
                imm: 64,
            }),
            decode(0b0000010_00000_01001_000_00001_1100111)?
        );

        // addi x1, x9, 64
        assert_eq!(
            Instruction::Addi(IType {
                rd: 1,
                rs1: 9,
                imm: 64,
            }),
            decode(0b0000010_00000_01001_000_00001_0010011)?
        );

        // slli x2, x6, 17
        assert_eq!(
            Instruction::Slli(IType {
                rd: 2,
                rs1: 6,
                imm: 17,
            }),
            decode(0b0000000_10001_00110_001_00010_0010011)?
        );

        // slti x3, x4, 16
        assert_eq!(
            Instruction::Slti(IType {
                rd: 3,
                rs1: 4,
                imm: 16,
            }),
            decode(0b0000000_10000_00100_010_00011_0010011)?
        );

        // sltiu x4, x19, 8
        assert_eq!(
            Instruction::Sltiu(IType {
                rd: 4,
                rs1: 19,
                imm: 8,
            }),
            decode(0b0000000_01000_10011_011_00100_0010011)?
        );

        // xori x5, x12, 4
        assert_eq!(
            Instruction::Xori(IType {
                rd: 5,
                rs1: 12,
                imm: 4,
            }),
            decode(0b0000000_00100_01100_100_00101_0010011)?
        );

        // srli x6, x17, 5
        assert_eq!(
            Instruction::Srli(IType {
                rd: 6,
                rs1: 17,
                imm: 5,
            }),
            decode(0b0000000_00101_10001_101_00110_0010011)?
        );

        // srai x7, x27, 1024
        assert_eq!(
            Instruction::Srai(IType {
                rd: 7,
                rs1: 27,
                imm: 1024,
            }),
            decode(0b0100000_00000_11011_101_00111_0010011)?
        );

        // ori x8, x13, 2
        assert_eq!(
            Instruction::Ori(IType {
                rd: 8,
                rs1: 13,
                imm: 2,
            }),
            decode(0b0000000_00010_01101_110_01000_0010011)?
        );

        // andi x9, x30, 1
        assert_eq!(
            Instruction::Andi(IType {
                rd: 9,
                rs1: 30,
                imm: 1,
            }),
            decode(0b0000000_00001_11110_111_01001_0010011)?
        );

        // lb x9, x30, 2
        assert_eq!(
            Instruction::Lb(IType {
                rd: 9,
                rs1: 30,
                imm: 2,
            }),
            decode(0b0000000_00010_11110_000_01001_0000011)?
        );

        // lh x9, x30, 1
        assert_eq!(
            Instruction::Lh(IType {
                rd: 9,
                rs1: 30,
                imm: 1,
            }),
            decode(0b0000000_00001_11110_001_01001_0000011)?
        );

        // lw x9, x30, 2048
        assert_eq!(
            Instruction::Lw(IType {
                rd: 9,
                rs1: 30,
                imm: 2048,
            }),
            decode(0b1000000_00000_11110_010_01001_0000011)?
        );

        // lbu x9, x30, 1
        assert_eq!(
            Instruction::Lbu(IType {
                rd: 9,
                rs1: 30,
                imm: 1,
            }),
            decode(0b0000000_00001_11110_100_01001_0000011)?
        );

        // lhu x9, x30, 1
        assert_eq!(
            Instruction::Lhu(IType {
                rd: 9,
                rs1: 30,
                imm: 1,
            }),
            decode(0b0000000_00001_11110_101_01001_0000011)?
        );

        // csrrw x1, 1024, x2
        assert_eq!(
            Instruction::Csrrw(IType {
                rd: 1,
                rs1: 2,
                imm: 1024
            }),
            decode(0b0100000_00000_00010_001_00001_1110011)?
        );

        // csrrs x1, 1024, x2
        assert_eq!(
            Instruction::Csrrs(IType {
                rd: 1,
                rs1: 2,
                imm: 1024
            }),
            decode(0b0100000_00000_00010_010_00001_1110011)?
        );

        // csrrc x1, 1024, x2
        assert_eq!(
            Instruction::Csrrc(IType {
                rd: 1,
                rs1: 2,
                imm: 1024
            }),
            decode(0b0100000_00000_00010_011_00001_1110011)?
        );

        // csrrwi x1, 1024, x2
        assert_eq!(
            Instruction::Csrrwi(IType {
                rd: 1,
                rs1: 2,
                imm: 1024
            }),
            decode(0b0100000_00000_00010_101_00001_1110011)?
        );

        // csrrsi x1, 1024, x2
        assert_eq!(
            Instruction::Csrrsi(IType {
                rd: 1,
                rs1: 2,
                imm: 1024
            }),
            decode(0b0100000_00000_00010_110_00001_1110011)?
        );

        // csrrci x1, 1024, x2
        assert_eq!(
            Instruction::Csrrci(IType {
                rd: 1,
                rs1: 2,
                imm: 1024
            }),
            decode(0b0100000_00000_00010_111_00001_1110011)?
        );
        Ok(())
    }

    #[test]
    fn decode_invalid_rv32i_i() -> Result<(), Exception> {
        // jalr x1, x9, 65
        assert_eq!(
            Err(Exception::InstructionAddressMisaligned),
            decode(0b0000010_00001_01001_000_00001_1100111)
        );
        Ok(())
    }

    #[test]
    fn decode_rv32i_s() -> Result<(), Exception> {
        // sb x1, x2, 2899
        assert_eq!(
            Instruction::Sb(SType {
                rs1: 1,
                rs2: 2,
                imm: 2899
            }),
            decode(0b1011010_00010_00001_000_10011_0100011)?
        );

        // sh x1, x2, 1397
        assert_eq!(
            Instruction::Sh(SType {
                rs1: 1,
                rs2: 2,
                imm: 1397
            }),
            decode(0b0101011_00010_00001_001_10101_0100011)?
        );

        // sw x1, x2, 1397
        assert_eq!(
            Instruction::Sw(SType {
                rs1: 1,
                rs2: 2,
                imm: 1397
            }),
            decode(0b0101011_00010_00001_010_10101_0100011)?
        );
        Ok(())
    }

    #[test]
    fn decode_rv32i_b() -> Result<(), Exception> {
        // beq x1, x2, 2048
        assert_eq!(
            Instruction::Beq(BType {
                rs1: 1,
                rs2: 2,
                imm: 2048,
            }),
            decode(0b0000000_00010_00001_000_00001_1100011)?
        );

        // bne x1, x2, 1024
        assert_eq!(
            Instruction::Bne(BType {
                rs1: 1,
                rs2: 2,
                imm: 1024,
            }),
            decode(0b0100000_00010_00001_001_00000_1100011)?
        );

        // blt x1, x2, 1024
        assert_eq!(
            Instruction::Blt(BType {
                rs1: 1,
                rs2: 2,
                imm: 1024,
            }),
            decode(0b0100000_00010_00001_100_00000_1100011)?
        );

        // bge x1, x2, 1024
        assert_eq!(
            Instruction::Bge(BType {
                rs1: 1,
                rs2: 2,
                imm: 1024,
            }),
            decode(0b0100000_00010_00001_101_00000_1100011)?
        );

        // bltu x1, x2, 1024
        assert_eq!(
            Instruction::Bltu(BType {
                rs1: 1,
                rs2: 2,
                imm: 1024,
            }),
            decode(0b0100000_00010_00001_110_00000_1100011)?
        );

        // bgeu x1, x2, 1024
        assert_eq!(
            Instruction::Bgeu(BType {
                rs1: 1,
                rs2: 2,
                imm: 1024,
            }),
            decode(0b0100000_00010_00001_111_00000_1100011)?
        );
        Ok(())
    }

    #[test]
    fn decode_rv32i_j() -> Result<(), Exception> {
        assert_eq!(
            Instruction::Jal(JType {
                rd: 1,
                imm: 0b010000001010000000000
            }),
            decode(0b01000000000010000001_00001_1101111)?
        );

        // jal x1, 4
        assert_eq!(
            Instruction::Jal(JType { rd: 0, imm: 4 }),
            decode(0b00000000010000000000_00000_1101111)?
        );

        // jal x1, -4
        assert_eq!(
            Instruction::Jal(JType {
                rd: 1,
                imm: 0b111111111111111111100
            }),
            // 11111111111111111100
            decode(0b11111111110111111111_00001_1101111)?
        );
        Ok(())
    }

    #[test]
    fn decode_rv32_u() -> Result<(), Exception> {
        // lui x1, 623706
        assert_eq!(
            Instruction::Lui(UType {
                rd: 1,
                imm: 2554699776,
            }),
            decode(0b10011000010001011010_00001_0110111)?
        );

        // auipc x1, 103275
        assert_eq!(
            Instruction::Auipc(UType {
                rd: 1,
                imm: 423014400,
            }),
            decode(0b00011001001101101011_00001_0010111)?
        );
        Ok(())
    }
}
