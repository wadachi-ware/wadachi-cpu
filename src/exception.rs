#[derive(Debug, PartialEq, Eq)]
pub enum Exception {
    InstructionAddressMisaligned,
    IllegalInstruction,
}
