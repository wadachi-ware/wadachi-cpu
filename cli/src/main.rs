use wadachi_cpu::emulator::Emulator;

fn main() {
    let mut emulator = Emulator::new();
    emulator.execute();
}
