pub mod decode;
pub mod exception;
pub mod memory;
pub mod processor;

#[cfg(test)]
mod tests {
    use crate::memory::{Memory, VectorMemory};
    use crate::processor::Processor;

    #[test]
    fn register_caluculation() {
        /*
        00178793 addi a5,a5,1
        00278793 addi a5,a5,2
        00380813 addi a6,a6,3
        00281813 slli a6,a6,0x2
        010787b3 add a5,a5,a6
        */
        let memory = vec![0; 24];
        let memory: Box<dyn Memory> = Box::new(VectorMemory::from(memory));
        let entry_point = 4;
        let mut processor = Processor::new(memory);
        processor.set_pc(entry_point);
        processor.load_raw(
            entry_point,
            vec![0x00178793, 0x00278793, 0x00380813, 0x00281813, 0x010787b3],
        );
        processor.execute();
        assert_eq!(15, processor.regs[15]);
        assert_eq!(12, processor.regs[16]);
    }
}
