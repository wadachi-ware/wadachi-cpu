use crate::exception::Exception;
use crate::processor::Mode;
use bit_field::BitField;
use std::ops::Range;

const CSR_NUM: usize = 4096;

const MODE_RANGE: Range<usize> = 8..10;
const READ_WRITE_RANGE: Range<usize> = 10..12;

pub mod address {
    /// User level CSRs.
    pub const USTATUS: usize = 0x000;
    pub const UIE: usize = 0x004;
    pub const UTVEC: usize = 0x005;
    pub const USCRATCH: usize = 0x040;
    pub const UEPC: usize = 0x041;
    pub const UCAUSE: usize = 0x042;
    pub const UTVAL: usize = 0x043;
    pub const UIP: usize = 0x044;

    /// Supervisor level CSRs.
    pub const SSTATUS: usize = 0x100;
    pub const SEDELEG: usize = 0x102;
    pub const SIDELEG: usize = 0x103;
    pub const SIE: usize = 0x104;
    pub const STVEC: usize = 0x105;
    pub const SCOUNTEREN: usize = 0x106;
    pub const SSCRATCH: usize = 0x140;
    pub const SEPC: usize = 0x141;
    pub const SCAUSE: usize = 0x142;
    pub const STVAL: usize = 0x143;
    pub const SIP: usize = 0x144;
    pub const SATP: usize = 0x180;

    /// Machine level CSRs.
    pub const MSTATUS: usize = 0x300;
    pub const MEDELEG: usize = 0x302;
    pub const MIDELEG: usize = 0x303;
    pub const MIE: usize = 0x304;
    pub const MTVEC: usize = 0x305;
    pub const MCOUNTEREN: usize = 0x306;
    pub const MSCRATCH: usize = 0x340;
    pub const MEPC: usize = 0x341;
    pub const MCAUSE: usize = 0x342;
    pub const MTVAL: usize = 0x343;
    pub const MIP: usize = 0x344;
}

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
