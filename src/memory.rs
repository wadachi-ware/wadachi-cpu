pub trait Memory {
    fn read_inst(&self, addr: u32) -> u32;

    fn read_word(&self, addr: u32) -> u32;
    fn write_word(&mut self, addr: u32, data: u32);
}

#[derive(Debug)]
pub struct VectorMemory {
    memory: Vec<u8>
}

impl VectorMemory {
    pub fn new(size: u32) -> Self {
        let mut memory = Vec::with_capacity(size);
        memory.resize(size, 0);

        Self {
            memory
        }
    }

    /// read big-endian word located at *addr*
    fn read_bw(&self, addr: u32) -> u32 {
        (self.memory[addr] as u32) << 24 |
        (self.memory[addr+1] as u32) << 16 |
        (self.memory[addr+2] as u32) << 8 |
        (self.memory[addr+3] as u32)
    }

    /// read little-endian word located at *addr*
    fn read_lw(&self, addr: u32) -> u32 {
        (self.memory[addr] as u32) |
        (self.memory[addr+1] as u32) << 8 |
        (self.memory[addr+2] as u32) << 16 |
        (self.memory[addr+3] as u32) << 24
    }

    /// write big-endian word at *addr*
    fn write_bw(&mut self, addr: u32, val: u32) {
        self.memory[addr] = (val >> 24) as u8;
        self.memory[addr+1] = (val >> 16) as u8;
        self.memory[addr+2] = (val >> 8) as u8;
        self.memory[addr+3] = val as u8;
    }
    
    /// write little-endian word at *addr*
    fn write_lw(&mut self, addr: u32, val: u32) {
        self.memory[addr] = val as u8;
        self.memory[addr+1] = (val >> 8) as u8;
        self.memory[addr+2] = (val >> 16) as u8;
        self.memory[addr+3] = (val >> 24) as u8;
    }

    /// read an instruction located at addr
    pub fn write_inst(&mut self, addr: u32, inst: u32) {
        self.write_bw(addr, inst);
    }
}

impl Memory for VectorMemory {
    /// read an instruction located at *addr*
    fn read_inst(&self, addr: u32) -> u32 {
        self.read_bw(addr)
    }

    /// read word located at *addr*
    fn read_word(&self, addr: u32) -> u32 {
        self.read_lw(addr)
    }

    /// write word at *addr*
    fn write_word(&mut self, addr: u32, data: u32) {
        self.write_lw(addr, data);
    }
}

impl From<Vec<u8>> for VectorMemory {
    fn from(memory: Vec<u8>) -> Self {
        Self {
            memory
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_memory() {
        let mut mem = VectorMemory::new(16);

        assert_eq!(mem.read_word(0), 0);
        assert_eq!(mem.read_word(4), 0);
        assert_eq!(mem.read_word(8), 0);
        assert_eq!(mem.read_word(12), 0);

        mem.write_word(0, 0x12345678);
        mem.write_word(4, 0x90abcdef);
        mem.write_word(8, 0xdeadbeef);
        mem.write_word(12, 0xabadbabe);

        assert_eq!(mem.read_word(0), 0x12345678);
        assert_eq!(mem.read_word(4), 0x90abcdef);
        assert_eq!(mem.read_word(8), 0xdeadbeef);
        assert_eq!(mem.read_word(12), 0xabadbabe);
    }
}
