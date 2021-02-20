use crate::memory::{Memory, VectorMemory};
use crate::processor::Processor;

/// Struct to set up and hold execution environment.
pub struct Emulator {
    processor: Processor,
}

impl Emulator {
    pub fn new() -> Self {
        let memory = vec![];
        let memory: Box<dyn Memory> = Box::new(VectorMemory::from(memory));
        let processor = Processor::new(memory);
        Self { processor }
    }

    pub fn execute(&mut self) {
        loop {
            if let Err(err) = self.processor.tick() {
                unimplemented!();
            }
        }
    }
}
