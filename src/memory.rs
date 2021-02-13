pub trait Memory {
    fn read_inst(&self, addr: usize) -> u32;

    fn read_word(&self, addr: usize) -> u32;
    fn write_word(&mut self, addr: usize, data: u32);
}

#[derive(Debug)]
pub struct EmptyMemory;

impl Memory for EmptyMemory {
    fn read_inst(&self, _addr: usize) -> u32 {
        0
    }

    fn read_word(&self, _addr: usize) -> u32 {
        0
    }

    fn write_word(&mut self, _addr: usize, _data: u32) {
    }
}

#[derive(Debug)]
pub struct VectorMemory {
    memory: Vec<u8>
}

impl VectorMemory {
    pub fn new(size: usize) -> Self {
        let mut memory = Vec::with_capacity(size);
        memory.resize(size, 0);

        Self {
            memory
        }
    }

    /// read big-endian word located at *addr*
    fn read_bw(&self, addr: usize) -> u32 {
        (self.memory[addr] as u32) << 24 |
        (self.memory[addr+1] as u32) << 16 |
        (self.memory[addr+2] as u32) << 8 |
        (self.memory[addr+3] as u32)
    }

    /// read little-endian word located at *addr*
    fn read_lw(&self, addr: usize) -> u32 {
        (self.memory[addr] as u32) |
        (self.memory[addr+1] as u32) << 8 |
        (self.memory[addr+2] as u32) << 16 |
        (self.memory[addr+3] as u32) << 24
    }

    /// write big-endian word at *addr*
    fn write_bw(&mut self, addr: usize, val: u32) {
        self.memory[addr] = (val >> 24) as u8;
        self.memory[addr+1] = (val >> 16) as u8;
        self.memory[addr+2] = (val >> 8) as u8;
        self.memory[addr+3] = val as u8;
    }
    
    /// write little-endian word at *addr*
    fn write_lw(&mut self, addr: usize, val: u32) {
        self.memory[addr] = val as u8;
        self.memory[addr+1] = (val >> 8) as u8;
        self.memory[addr+2] = (val >> 16) as u8;
        self.memory[addr+3] = (val >> 24) as u8;
    }

    /// read an instruction located at addr
    pub fn write_inst(&mut self, addr: usize, inst: u32) {
        self.write_bw(addr, inst);
    }
}

impl Memory for VectorMemory {
    /// read an instruction located at *addr*
    fn read_inst(&self, addr: usize) -> u32 {
        self.read_bw(addr)
    }

    /// read word located at *addr*
    fn read_word(&self, addr: usize) -> u32 {
        self.read_lw(addr)
    }

    /// write word at *addr*
    fn write_word(&mut self, addr: usize, data: u32) {
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
    fn empty_memory() {
        let mut mem = EmptyMemory;

        assert_eq!(mem.read_word(0), 0);
        assert_eq!(mem.read_word(4), 0);
        assert_eq!(mem.read_word(8), 0);
        assert_eq!(mem.read_word(12), 0);

        mem.write_word(0, 0x12345678);
        mem.write_word(4, 0x90abcdef);
        mem.write_word(8, 0xdeadbeef);
        mem.write_word(12, 0xabadbabe);

        assert_eq!(mem.read_word(0), 0);
        assert_eq!(mem.read_word(4), 0);
        assert_eq!(mem.read_word(8), 0);
        assert_eq!(mem.read_word(12), 0);
    }

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
