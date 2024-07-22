mod bitreader;

use std::{env, fs};
use std::fs::File;
use std::io::{self, Read, Write, Cursor, Seek, SeekFrom};
use std::path::Path;
use crate::bitreader::BitReader;

fn read_file_to_byte_vector(file_path: &Path) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}

fn main() -> io::Result<()>{
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.deflate>", args[0]);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Incorrect number of arguments"));
    }

    let file_path = Path::new(&args[1]);
    let data = read_file_to_byte_vector(file_path)?;

    let mut bit_reader = BitReader::new(&data);
    // assert!(bit_reader.read_bits(1, false)? == 1);
    // bit_reader.read_bits(1, false)?;
    // let btype = bit_reader.read_bits(2, true)?;

    // remove .deflate extension
    let output_file_name = file_path.file_stem().unwrap().to_str().unwrap();

    let decoded_results = bit_reader.read_bitstream_blocks()?;
        
    let mut output_file = File::create(output_file_name)?;
    output_file.write_all(&decoded_results)?;


    // if btype == 1 { // fixed huffman
    //     let decoded_results = bit_reader.read_bitstream_fixed_huffman()?;
        
    //     let mut output_file = File::create(output_file_name)?;
    //     output_file.write_all(&decoded_results)?;

    // } else { // dynamic huffman
        
        
    // }

    // // test for read_bitstream()
    // println!("Bits: ");
    // for bit in bits {
    //     print!("{}", if bit {1} else {0});
    // }    
    // println!();
    Ok(())
}
