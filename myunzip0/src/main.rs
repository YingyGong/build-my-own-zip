use std::{env, fs};
use std::fs::File;
use std::io::{self, Read, Write, Cursor, Seek, SeekFrom};
use std::path::Path;

fn read_u16_le(cursor: &mut Cursor<&[u8]>) -> io::Result<u16> {
    let mut bytes: [u8; 2] = [0u8; 2];
    cursor.read_exact(&mut bytes)?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_u32_le(cursor: &mut Cursor<&[u8]>) -> io::Result<u32> {
    let mut bytes = [0u8; 4];
    cursor.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn find_eocd(buffer: &[u8]) -> Option<usize> {
    let eocd_signature: [u8; 4] = [0x50, 0x4B, 0x05, 0x06];
    buffer.windows(4).rposition(|window| window == eocd_signature)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.zip>", args[0]);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Incorrect number of arguments"));
    }

    let zip_file_path = &args[1];
    let mut zip_file = File::open(zip_file_path)?;
    let mut buffer = Vec::new();

    zip_file.read_to_end(&mut buffer)?;
    let mut cursor = Cursor::new(&buffer[..]);

    let signature = read_u32_le(&mut cursor)?;
    if signature != 0x04034b50 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Not a Zip File"));
    }

    cursor.seek(SeekFrom::Current(4))?;
    let compression_method = read_u16_le(&mut cursor)?;
    assert!(compression_method == 0 || compression_method == 8);

    cursor.seek(SeekFrom::Current(16))?;

    let file_name_length = read_u16_le(&mut cursor)?;
    let extra_field_length = read_u16_le(&mut cursor)?;

    let mut file_name_bytes = vec![0; file_name_length as usize];
    cursor.read_exact(&mut file_name_bytes)?;
    let file_name = file_name_bytes.iter()
        .map(|&b| if b.is_ascii() { b as char } else { '?' })
        .collect::<String>();
    
    cursor.seek(SeekFrom::Current(extra_field_length as i64))?;
    let start_of_data = cursor.position() as usize;

    if let Some(eocd_pos) = find_eocd(&buffer) {

        let mut cursor: Cursor<&[u8]> = Cursor::new(&buffer[eocd_pos..]);
        cursor.seek(SeekFrom::Current(16))?;
        let offset_of_start_of_central_directory = read_u32_le(&mut cursor).unwrap() as usize;
        let start_of_central_directory = buffer[cursor.position() as usize..]
        .windows(4)
        .position(|window| window == &[0x50, 0x4B, 0x01, 0x02])
        .unwrap_or_else(|| buffer.len()) + cursor.position() as usize;

        assert!(offset_of_start_of_central_directory == start_of_central_directory);

        let file_data = &buffer[start_of_data..offset_of_start_of_central_directory];
        let output_file_name = match compression_method {
            0 => file_name,
            8 => format!("{}.deflate", file_name),
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported compression method")),
        };

        let path = Path::new(&output_file_name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
    
        let mut output_file = File::create(path)?;
        output_file.write_all(file_data)?;

    } else {
        println!("EOCD not found - not a ZIP file or corrupted.");
    }

    Ok(())
}