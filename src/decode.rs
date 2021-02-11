use bit_field::BitField;

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Add(RType),
}

#[derive(Debug, PartialEq, Eq)]
pub struct RType {
    rd: u8,
    rs1: u8,
    rs2: u8,
}

pub fn decode(instruction: u32) -> Instruction {
    match instruction.get_bits(0..7) {
        0b0110011 => {
            match (instruction.get_bits(12..15), instruction.get_bits(25..32)) {
                (0b000, 0b0000000) => Instruction::Add(RType {
                    rd: instruction.get_bits(7..12) as u8,
                    rs1: instruction.get_bits(15..20) as u8,
                    rs2: instruction.get_bits(20..25) as u8,
                }),
                _ => unimplemented!(),
            }
        }
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
    }
}
