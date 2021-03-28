use std::io::{self, Read};
use std::{fs::File, path::PathBuf};
use structopt::StructOpt;
use wadachi_cpu::{self, memory::VectorMemory, processor::Processor};

#[derive(StructOpt)]
struct Opt {
    /// Time interval to execute every instruction in millisec
    #[structopt(long, short, default_value)]
    interval: u64,

    /// If specified, dump register values at the end of execution
    #[structopt(long, short)]
    verbose: bool,

    /// Size of main memory in hex.
    #[structopt(long, short, parse(try_from_str = parse_hex))]
    size: usize,

    /// File path to kernel
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn parse_hex(src: &str) -> Result<usize, std::num::ParseIntError> {
    let src = src.strip_prefix("0x").unwrap_or(src);
    usize::from_str_radix(src, 16)
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    let mut file = File::open(opt.file)?;
    let mut program = Vec::new();
    file.read_to_end(&mut program)?;

    let memory = VectorMemory::new(opt.size);
    let mut processor = Processor::new(Box::new(memory));
    processor.set_interval(opt.interval);
    if let Err(err) = processor.load_elf(program) {
        eprintln!("{:?}", err);
    }
    processor.execute();

    if opt.verbose {
        println!("{}", processor);
    }
    Ok(())
}
