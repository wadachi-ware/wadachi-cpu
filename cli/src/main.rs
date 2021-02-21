use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use wadachi_cpu::memory::{Memory, VectorMemory};
use wadachi_cpu::processor::Processor;

fn main() {
    let memory_size = 1024;
    let memory = vec![0; memory_size];
    let memory: Box<dyn Memory> = Box::new(VectorMemory::from(memory));

    let mut processor = Processor::new(memory);
    let start_address = 0;
    processor.set_pc(start_address);

    let program = match read_program("sample.bin") {
        Ok(program) => program,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };
    processor.load(start_address, program);
    processor.execute();
    assert_eq!(15, processor.regs[15]);
    assert_eq!(12, processor.regs[16]);
}

/// Read bytes from file and parse as an array of instructions.
fn read_program<P: AsRef<Path>>(file_name: P) -> io::Result<Vec<u32>> {
    let mut file = File::open(file_name)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer
        .chunks_exact(4)
        .map(|bytes| deserialize_big_endian(bytes))
        .collect())
}

/// Deserialize 4byte integers into an instruction.
/// TODO: this function should be defined in `wadachi_cpu` crate.
fn deserialize_big_endian(bytes: &[u8]) -> u32 {
    assert_eq!(4, bytes.len());
    (bytes[0] as u32) << 24 | (bytes[1] as u32) << 16 | (bytes[2] as u32) << 8 | (bytes[3] as u32)
}
