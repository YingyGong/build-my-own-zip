mod bitwriter;
mod lz77;

use std::env;
use std::fs::File;
use std::io::{Write, BufReader, Read};
use crate::bitwriter::BitWriter;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <output.zip> <inputfile>", args[0]);
        std::process::exit(1);
    }

    let output_zip = &args[1];
    let input_file = &args[2];

    let mut file = BufReader::new(File::open(input_file)?);
    let mut file_data = Vec::new();
    file.read_to_end(&mut file_data)?;

    let uncompressed_file_size = file_data.len() as u32;
    let file_name = input_file.as_bytes();
    let file_name_length = file_name.len() as u16;

    let mut bit_writer = BitWriter::new();
    bit_writer.write_bitstream_fixed_huffman(&file_data)?;
    let compressed_file_data = bit_writer.get_buffer();
    let compressed_file_size = compressed_file_data.len() as u32;

    let mut local_header = Vec::new();
    local_header.extend(&0x04034b50u32.to_le_bytes());
    local_header.extend(&20u16.to_le_bytes());
    local_header.extend(&0u16.to_le_bytes());
    local_header.extend(&8u16.to_le_bytes()); //use deflate to compress
    local_header.extend(&0u16.to_le_bytes());
    local_header.extend(&0u16.to_le_bytes());
    local_header.extend(&0xdeadbeefu32.to_le_bytes());
    local_header.extend(&compressed_file_size.to_le_bytes());
    local_header.extend(&uncompressed_file_size.to_le_bytes());
    local_header.extend(&file_name_length.to_le_bytes());
    local_header.extend(&0u16.to_le_bytes());
    local_header.extend(file_name);

    // file data added in the end

    let mut central_dir = Vec::new();
    central_dir.extend(&0x02014b50u32.to_le_bytes());
    central_dir.extend(&30u8.to_le_bytes());
    central_dir.extend(&65u8.to_le_bytes());
    central_dir.extend(&20u16.to_le_bytes());
    central_dir.extend(&0u16.to_le_bytes());
    central_dir.extend(&8u16.to_le_bytes()); //use deflate to compress
    central_dir.extend(&0u16.to_le_bytes());
    central_dir.extend(&0u16.to_le_bytes());
    central_dir.extend(&0xdeadbeefu32.to_le_bytes());
    central_dir.extend(&compressed_file_size.to_le_bytes());
    central_dir.extend(&uncompressed_file_size.to_le_bytes());
    central_dir.extend(&file_name_length.to_le_bytes());
    central_dir.extend(&0u16.to_le_bytes());
    central_dir.extend(&0u16.to_le_bytes());
    central_dir.extend(&0u16.to_le_bytes());
    central_dir.extend(&1u16.to_le_bytes());
    central_dir.extend(&1u32.to_le_bytes());
    central_dir.extend(&0u32.to_le_bytes());
    central_dir.extend(file_name);

    let mut end_central_dir = Vec::new();
    end_central_dir.extend(&0x06054b50u32.to_le_bytes());
    end_central_dir.extend(&0u16.to_le_bytes());
    end_central_dir.extend(&0u16.to_le_bytes());
    end_central_dir.extend(&1u16.to_le_bytes());
    end_central_dir.extend(&1u16.to_le_bytes());
    end_central_dir.extend(&(central_dir.len() as u32).to_le_bytes());
    end_central_dir.extend(&(local_header.len() as u32 + compressed_file_size).to_le_bytes());
    end_central_dir.extend(&0u16.to_le_bytes());

    let mut output = File::create(output_zip)?;
    output.write_all(&local_header)?;
    output.write_all(&compressed_file_data)?;
    output.write_all(&central_dir)?;
    output.write_all(&end_central_dir)?;

    Ok(())
}
