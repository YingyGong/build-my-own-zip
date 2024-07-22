mod bitwriter;
mod lz77;

use std::{env, fs};
use std::fs::File;
use std::io::{self, Read, Write, Cursor, Seek, SeekFrom};
use std::path::Path;
use crate::bitwriter::BitWriter;

fn read_file_to_byte_vector(file_path: &Path) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}

fn main() -> io::Result<()>{
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input>", args[0]);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Incorrect number of arguments"));
    }

    let file_path = Path::new(&args[1]);
    let data: Vec<u8> = read_file_to_byte_vector(file_path)?;

    let mut bit_writer = BitWriter::new();

    // add .deflate extension
    // let output_file_name: std::path::PathBuf = file_path.with_extension("deflate");
    let output_file_name = file_path.to_owned().into_os_string().into_string().unwrap() + ".deflate";

    bit_writer.write_bitstream_fixed_huffman(&data)?;

    // i haven't handle empty file yet, do I need to?
    let mut output_file = File::create(Path::new(&output_file_name))?;

    // write nothing in the file
    output_file.write_all(&bit_writer.get_buffer())?;
        
    Ok(())
}
