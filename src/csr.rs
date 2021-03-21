use crate::exception::Exception;
use crate::processor::Mode;
use bit_field::BitField;
use std::ops::Range;

const CSR_NUM: usize = 4096;

const MODE_RANGE: Range<usize> = 8..10;
const READ_WRITE_RANGE: Range<usize> = 10..12;

#[derive(Clone, Debug)]
pub struct Csr {
    registers: [u32; CSR_NUM],
}

impl Csr {
    pub fn new() -> Self {
        Self {
            registers: [0; CSR_NUM],
        }
    }

    #[inline]
    fn is_valid_addr(addr: usize) -> Result<(), Exception> {
        if addr >= CSR_NUM {
            Err(Exception::IllegalInstruction)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn is_valid_mode(addr: usize, mode: Mode) -> Result<(), Exception> {
        if (mode as usize) < addr.get_bits(MODE_RANGE) {
            // cf. RISC-V Unprivileged ISA V20191213, P5
            Err(Exception::IllegalInstruction)
        } else {
            Ok(())
        }
    }

    pub fn read(&self, addr: usize, mode: Mode) -> Result<u32, Exception> {
        Self::is_valid_addr(addr)?;
        Self::is_valid_mode(addr, mode)?;
        Ok(self.registers[addr])
    }

    pub fn write(&mut self, addr: usize, value: u32, mode: Mode) -> Result<(), Exception> {
        Self::is_valid_addr(addr)?;
        Self::is_valid_mode(addr, mode)?;
        // If the CSR is readonly, the write is ignored.
        if addr.get_bits(READ_WRITE_RANGE) == 0b11 {
            return Ok(());
        }
        self.registers[addr] = value;
        Ok(())
    }
}

impl Default for Csr {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_and_write_csr() -> Result<(), Exception> {
        let mut csr = Csr::new();
        // User Read/Write.
        let addr = 0x000;
        let value = 0x1;
        csr.write(addr, value, Mode::User)?;
        assert_eq!(value, csr.read(addr, Mode::User)?);
        Ok(())
    }

    #[test]
    fn read_csr_invalid_mode() -> Result<(), Exception> {
        let mut csr = Csr::new();
        // Machine Read/Write.
        let addr = 0x300;
        let value = 0x1;
        csr.write(addr, value, Mode::Machine)?;
        assert_eq!(
            Err(Exception::IllegalInstruction),
            csr.read(addr, Mode::User)
        );
        assert_eq!(
            Err(Exception::IllegalInstruction),
            csr.read(addr, Mode::Supervisor)
        );
        Ok(())
    }

    #[test]
    fn write_csr_invalid_mode() -> Result<(), Exception> {
        let mut csr = Csr::new();
        // Machine Read/Write.
        let addr = 0x300;
        let value = 0x1;
        assert_eq!(
            Err(Exception::IllegalInstruction),
            csr.write(addr, value, Mode::User)
        );
        assert_eq!(
            Err(Exception::IllegalInstruction),
            csr.write(addr, value, Mode::Supervisor)
        );
        Ok(())
    }

    #[test]
    fn write_readonly_csr() -> Result<(), Exception> {
        let mut csr = Csr::new();
        // Machine Readonly.
        let addr = 0xf00;
        let value = 0x1;
        csr.write(addr, value, Mode::Machine)?;
        // Write to a readonly register is ignored.
        assert_eq!(0, csr.read(addr, Mode::Machine)?);
        Ok(())
    }
}
