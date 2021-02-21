use wadachi_cpu::emulator::Emulator;

// parse file as a sequence of u32 integer
// program execution
fn main() {
    let mut emulator = Emulator::new();
    emulator.execute();
}
