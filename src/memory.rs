pub trait Memory {
    /// Read an instruction located at *addr*
    fn read_inst(&self, addr: usize) -> u32;

    /// Read byte located at *addr*
    fn read_byte(&self, addr: usize) -> u8;
    
    /// Read half word located at *addr*
    fn read_halfword(&self, addr: usize) -> u16;

    /// Read word located at *addr*
    fn read_word(&self, addr: usize) -> u32;

    /// Write an instruction located at *addr*
    fn write_inst(&mut self, addr: usize, data: u32);

    /// Write halfword at *addr*
    fn write_byte(&mut self, addr: usize, data: u8);

    /// Write halfword at *addr*
    fn write_halfword(&mut self, addr: usize, data: u16);

    /// Write word at *addr*
    fn write_word(&mut self, addr: usize, data: u32);

    /// Get memory size in byte.
    fn len(&self) -> usize;
}

#[derive(Debug)]
pub struct EmptyMemory;

impl Memory for EmptyMemory {
    fn read_inst(&self, _addr: usize) -> u32 {
        0
    }

    fn read_byte(&self, _addr: usize) -> u8 {
        0
    }

    fn read_halfword(&self, _addr: usize) -> u16 {
        0
    }

    fn read_word(&self, _addr: usize) -> u32 {
        0
    }
  
    fn write_inst(&mut self, _addr: usize, _data: u32) {}

    fn write_byte(&mut self, _addr: usize, _data: u8) {}

    fn write_halfword(&mut self, _addr: usize, _data: u16) {}

    fn write_word(&mut self, _addr: usize, _data: u32) {}

    fn len(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub struct VectorMemory {
    memory: Vec<u8>,
}

impl VectorMemory {
    pub fn new(size: usize) -> Self {
        let mut memory = Vec::with_capacity(size);
        memory.resize(size, 0);

        Self { memory }
    }

    /// read little-endian byte located at *addr*
    fn read_lb(&self, addr: usize) -> u8 {
        self.memory[addr]
    }

    /// read little-endian half word located at *addr*
    fn read_lh(&self, addr: usize) -> u16 {
        (self.memory[addr] as u16) | (self.memory[addr + 1] as u16) << 8
    }

    /// read big-endian word located at *addr*
    fn read_bw(&self, addr: usize) -> u32 {
        (self.memory[addr] as u32) << 24
            | (self.memory[addr + 1] as u32) << 16
            | (self.memory[addr + 2] as u32) << 8
            | (self.memory[addr + 3] as u32)
    }

    /// read little-endian word located at *addr*
    fn read_lw(&self, addr: usize) -> u32 {
        (self.memory[addr] as u32)
            | (self.memory[addr + 1] as u32) << 8
            | (self.memory[addr + 2] as u32) << 16
            | (self.memory[addr + 3] as u32) << 24
    }

    /// Write little-endian byte located at *addr*
    fn write_lb(&mut self, addr: usize, val: u8) {
        self.memory[addr] = val;
    }

    /// Write little-endian halfword located at *addr*
    fn write_lh(&mut self, addr: usize, val: u16) {
        self.memory[addr] = val as u8;
        self.memory[addr + 1] = (val >> 8) as u8;
    }

    /// write big-endian word at *addr*
    fn write_bw(&mut self, addr: usize, val: u32) {
        self.memory[addr] = (val >> 24) as u8;
        self.memory[addr + 1] = (val >> 16) as u8;
        self.memory[addr + 2] = (val >> 8) as u8;
        self.memory[addr + 3] = val as u8;
    }

    /// write little-endian word at *addr*
    fn write_lw(&mut self, addr: usize, val: u32) {
        self.memory[addr] = val as u8;
        self.memory[addr + 1] = (val >> 8) as u8;
        self.memory[addr + 2] = (val >> 16) as u8;
        self.memory[addr + 3] = (val >> 24) as u8;
    }

    /// read an instruction located at addr
    /// This impl stores instructions as big-endian value
    /// but, we don't know whether it's popular...
    pub fn write_inst(&mut self, addr: usize, inst: u32) {
        self.write_bw(addr, inst);
    }
}

impl Memory for VectorMemory {
    fn read_inst(&self, addr: usize) -> u32 {
        self.read_bw(addr)
    }

    fn read_byte(&self, addr: usize) -> u8 {
        self.read_lb(addr)
    }

    fn read_halfword(&self, addr: usize) -> u16 {
        self.read_lh(addr)
    }

    fn read_word(&self, addr: usize) -> u32 {
        self.read_lw(addr)
    }

    /// write word at *addr*
    fn write_inst(&mut self, addr: usize, data: u32) {
        self.write_bw(addr, data);
    }

    fn write_byte(&mut self, addr: usize, data: u8) {
        self.write_lb(addr, data);
    }

    fn write_halfword(&mut self, addr: usize, data: u16) {
        self.write_lh(addr, data);
    }

    fn write_word(&mut self, addr: usize, data: u32) {
        self.write_lw(addr, data);
    }

    fn len(&self) -> usize {
        self.memory.len()
    }
}

impl From<Vec<u8>> for VectorMemory {
    fn from(memory: Vec<u8>) -> Self {
        Self { memory }
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

        mem.write_byte(4, 0x78);
        mem.write_byte(5, 0x56);
        mem.write_byte(6, 0x34);
        mem.write_byte(7, 0x12);
        assert_eq!(mem.read_byte(4), 0x78);
        assert_eq!(mem.read_byte(5), 0x56);
        assert_eq!(mem.read_byte(6), 0x34);
        assert_eq!(mem.read_byte(7), 0x12);
        assert_eq!(mem.read_word(4), 0x12345678);

        mem.write_halfword(8, 0x5678);
        mem.write_halfword(10, 0x1234);
        assert_eq!(mem.read_halfword(8), 0x5678);
        assert_eq!(mem.read_halfword(10), 0x1234);
        assert_eq!(mem.read_word(8), 0x12345678);

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
