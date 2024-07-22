mod lz77;

use lz77::{LZ77, OutputBuffer};
use std::fs::File;
use std::io::{self, Read, BufReader};
use std::path::Path;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: lz77 <input file>");
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Incorrect arguments"));
    }

    let input_path = Path::new(&args[1]);
    let mut file = BufReader::new(File::open(input_path)?);
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let mut compressor = LZ77::new(32768, 258);
    let mut output_buffer = OutputBuffer::new();

    compressor.compress(&data, &mut output_buffer);

    let output_path = input_path.with_extension("lz77");
    output_buffer.write_to_file(&output_path)
}
