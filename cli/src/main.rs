use std::fs::File;
use std::io::{self, Read};

use wadachi_cpu::{self, memory::VectorMemory, processor::Processor};

fn main() -> io::Result<()> {
    let path = "wadachi-os";
    let mut file = File::open(path)?;
    let mut program = Vec::new();
    file.read_to_end(&mut program)?;

    let memory_size = 0xf000_0000;
    let memory = VectorMemory::new(memory_size);
    let mut processor = Processor::new(Box::new(memory));
    if let Err(err) = processor.load(program) {
        eprintln!("{:?}", err);
    }
    processor.execute();
    Ok(())
}
