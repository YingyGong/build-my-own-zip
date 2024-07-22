use core::num;
use std::{io, vec};
use std::{env, fs};
use std::fs::File;
use std::io::{Read, Write, Cursor, Seek, SeekFrom};
use std::path::Path;
use crate::lz77::LZ77;


fn read_file_to_byte_vector(file_path: &Path) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}

pub struct BitWriter {
    buffer: Vec<u8>,
    position: usize,
    vec_bool: Vec<bool>,
}

pub fn convert_to_fixed_huffman_code(real_value: u8) -> u16 {
    let real_value = real_value as u16;
    let mut fixed_huffman_code: u16 = 0;
    if real_value < 144 {
        fixed_huffman_code = real_value + 48;
    } else {
        fixed_huffman_code = real_value + 256;
    }
    fixed_huffman_code
}

pub fn convert_length_to_fixed_huffman_code(real_value: u16) -> (u16, u16, u16) {
    assert!(real_value >= 3 && real_value <= 258);
    if real_value < 11 {
        return (real_value - 3 + 257, 0, 0);
    } else if real_value < 19 {
        let code_value = (real_value - 11) / 2 + 265;
        return (code_value, 1, (real_value + 1) % 2);
    } else if real_value < 35 {
        let code_value = (real_value - 19) / 4 + 269;
        return (code_value, 2, (real_value - 19) % 4);
    } else if real_value < 67 {
        let code_value = (real_value - 35) / 8 + 273;
        return (code_value, 3, (real_value - 35) % 8);
    } else if real_value < 131 {
        let code_value = (real_value - 67) / 16 + 277;
        return (code_value, 4, (real_value - 67) % 16);
    } else if real_value < 258 {
        let code_value = (real_value - 131) / 32 + 281;
        return (code_value, 5, (real_value - 131) % 32);
    } else {
        assert!(real_value == 258);
        return (285, 0, 0);
    }
}

pub fn convert_length_to_fixed_huffman_code_step_2(code_value: u16) -> u16 {
    if code_value < 280 {
        return code_value - 256
    } else {
        return code_value - 88
    }
}

pub fn convert_distance_to_fixed_huffman_code(real_value: u16) -> (u16, u16, u16) {
    assert!(real_value >= 1);
    if real_value > 32768 {
        println!("Distance value is too large: {}", real_value);
    }
    assert!(real_value <= 32768);
    if real_value < 5 {
        return (real_value - 1, 0, 0);
    }
    let num_extra_bits = ((real_value - 1) as f32).log2().floor() as u32 - 1;
    let code_value = (num_extra_bits + 1) * 2 + ((real_value - 1) as u32 / (2 << (num_extra_bits - 1))) % 2;
    let mut extra_bits_value;
    if code_value % 2 == 0 {
        extra_bits_value = real_value - (2 << num_extra_bits) - 1;
    } else {
        extra_bits_value = real_value - (2 << num_extra_bits) - (2 << (num_extra_bits - 1)) - 1;
    }
    return (code_value as u16, num_extra_bits as u16, extra_bits_value);
}


pub fn get_fixed_huffman_code_length(real_value: u8) -> u8 {
    if real_value < 144 {
        return 8
    } else {
        return 9
    }
}

pub fn get_fixed_huffman_code_length_for_u16(code_value: u16) -> u8 {
    if code_value < 280 {
        return 7
    } else {
        return 8
    }
}



impl BitWriter {
    pub fn new() -> Self {
        BitWriter {
            buffer: Vec::new(),
            position: 0,
            vec_bool: Vec::new(),
        }
    }

    fn write_header(&mut self) {
        self.vec_bool.push(true);

        // 10 for fixed huffman (reverse order)
        self.vec_bool.push(true);
        self.vec_bool.push(false);
        self.position += 3;
    }

    fn write_single_general(&mut self, value: u16, count: u8, reverse: bool) -> io::Result<()> {
        for i in 0..count {
            if reverse {
                let cur_bit = (value >> i) & 1; // push the rightmost bit
                self.vec_bool.push(cur_bit == 1);
            } else {
                let cur_bit = (value >> (count - i - 1)) & 1; // push the leftmost bit
                self.vec_bool.push(cur_bit == 1);
            }
        }
        self.position += count as usize;
        Ok(())
    }

    pub fn write_single_literal(&mut self, value: u8) -> io::Result<()> {
        let count: u8 = get_fixed_huffman_code_length(value);
        let huffman_code = convert_to_fixed_huffman_code(value);
        self.write_single_general(huffman_code, count, false)
    }

    pub fn write_length(&mut self, length_value: u16) -> io::Result<()> {
        let (code_value, extra_bits, extra_value) = convert_length_to_fixed_huffman_code(length_value);
        let count = get_fixed_huffman_code_length_for_u16(code_value);
        let huffman_code = convert_length_to_fixed_huffman_code_step_2(code_value);
        self.write_single_general(huffman_code, count, false)?;

        if extra_bits != 0 {
            self.write_single_general(extra_value, extra_bits as u8, true)?;
        }
        Ok(())
    }

    pub fn write_distance(&mut self, distance_value: u16) -> io::Result<()> {
        let count = 5;
        let (code_value, extra_bits, extra_value) = convert_distance_to_fixed_huffman_code(distance_value);
        self.write_single_general(code_value, count, false)?;
        if extra_bits != 0 {
            println!("extra bits: {}, extra value: {}", extra_bits, extra_value);
            self.write_single_general(extra_value, extra_bits as u8, true)?;
        }
        Ok(())
    }

    // big endian
    // pub fn write_bitstream(&mut self) -> io::Result<()> {
    //     let mut byte: u8 = 0;
    //     let mut count: u8 = 0;
    //     for bit in &self.vec_bool {
    //         byte = byte << 1;
    //         if *bit {
    //             byte = byte | 1;
    //         }
    //         count += 1;
    //         if count == 8 {
    //             self.buffer.push(byte);
    //             byte = 0;
    //             count = 0;
    //         }
    //     }
    //     if count != 0 {
    //         byte = byte << (8 - count);
    //         self.buffer.push(byte);
    //     }
    //     Ok(())
    // }

    // little endian 
    fn write_bitstream_buffer_little_endian(&mut self) -> io::Result<()> {
        let mut byte: u8 = 0;
        let mut count: u8 = 0;
        for bit in &self.vec_bool {
            if *bit {
                byte |= 1 << count;  
            }
            count += 1;
            if count == 8 {
                self.buffer.push(byte);
                byte = 0;
                count = 0;
            }
        }
        if count != 0 {
            self.buffer.push(byte);  
        }
        Ok(())
    }
    

    fn write_eob(&mut self) -> io::Result<()> {
        for _ in 0..7 {
            self.vec_bool.push(false);
        }
        Ok(())
    }

    // the function to call
    pub fn write_bitstream_fixed_huffman(&mut self, data: &Vec<u8>) -> io::Result<()> {
        let mut compressor = LZ77::new(32768, 258);

        self.write_header();
        compressor.compress(data, self);
        
        self.write_eob()?;
        // check if the last byte is full
        if self.vec_bool.len() % 8 != 0 {
            let mut count = self.vec_bool.len() % 8;
            while count < 8 {
                self.vec_bool.push(false);
                count += 1;
            }
        }
        self.write_bitstream_buffer_little_endian()?;
        Ok(())
    }

    pub fn get_buffer(&self) -> &Vec<u8> {
        &self.buffer
    }

    pub fn get_vec_bool(&self) -> &Vec<bool> {
        &self.vec_bool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_to_fixed_huffman_code() {
        assert_eq!(convert_to_fixed_huffman_code(0), 48);
        assert_eq!(convert_to_fixed_huffman_code(143), 191);  
        assert_eq!(convert_to_fixed_huffman_code(144), 400);
        assert_eq!(convert_to_fixed_huffman_code(255), 511);
    }

    #[test]
    fn test_integer_division() {
        let a  = 16383 / (2 << 11);
        println!("{}", a);
    }
    #[test]
    fn test_get_fixed_huffman_code_length() {
        assert_eq!(get_fixed_huffman_code_length(0), 8);
        assert_eq!(get_fixed_huffman_code_length(143), 8);
        assert_eq!(get_fixed_huffman_code_length(144), 9);
        assert_eq!(get_fixed_huffman_code_length(255), 9);
    }

    #[test]
    fn test_write_bits_1() {
        let file_name = "../testdata/inflate/fixed-huffman-literals-expected";
        let file_path = Path::new(file_name);
        let data = read_file_to_byte_vector(file_path).unwrap();

        let mut bit_writer = BitWriter::new();
        bit_writer.write_bitstream_fixed_huffman(&data).unwrap();
        let buffer = bit_writer.get_buffer();
        println!("{:?}", buffer);
        let vec_bool = bit_writer.get_vec_bool();
        // convert vec_bool to vector of 0 and 1
        let mut vec_u8: Vec<u8> = Vec::new();
        for bit in vec_bool {
            if *bit {
                vec_u8.push(1);
            } else {
                vec_u8.push(0);
            }
        }
        println!("{:?}", vec_u8);
    }

    #[test]
    fn test_write_bits_2() {
        let file_name = "../testdata/inflate/fixed-huffman-empty-expected";
        let file_path = Path::new(file_name);
        let data = read_file_to_byte_vector(file_path).unwrap();

        let mut bit_writer = BitWriter::new();
        bit_writer.write_bitstream_fixed_huffman(&data).unwrap();
        let buffer = bit_writer.get_buffer();
        println!("{:?}", buffer);
        let vec_bool = bit_writer.get_vec_bool();
        // convert vec_bool to vector of 0 and 1
        let mut vec_u8: Vec<u8> = Vec::new();
        for bit in vec_bool {
            if *bit {
                vec_u8.push(1);
            } else {
                vec_u8.push(0);
            }
        }
        println!("{:?}", vec_u8);

    }

    #[test]
    fn test_write_bits_3() {
        let file_name = "../testdata/inflate/fixed-huffman-overlapping-run0-expected";
        let file_path = Path::new(file_name);
        let data = read_file_to_byte_vector(file_path).unwrap();

        let mut bit_writer = BitWriter::new();
        bit_writer.write_bitstream_fixed_huffman(&data).unwrap();
        let buffer = bit_writer.get_buffer();
        println!("{:?}", buffer);
        let vec_bool = bit_writer.get_vec_bool();
        // convert vec_bool to vector of 0 and 1
        let mut vec_u8: Vec<u8> = Vec::new();
        for bit in vec_bool {
            if *bit {
                vec_u8.push(1);
            } else {
                vec_u8.push(0);
            }
        }
        println!("{:?}", vec_u8);
    }

    #[test]
    fn test_write_bits_4() {
        let file_name = "../testdata/inflate/fixed-huffman-overlapping-run1-expected";
        let file_path = Path::new(file_name);
        let data = read_file_to_byte_vector(file_path).unwrap();

        let mut bit_writer = BitWriter::new();
        bit_writer.write_bitstream_fixed_huffman(&data).unwrap();
        let buffer = bit_writer.get_buffer();
        println!("{:?}", buffer);
        let vec_bool = bit_writer.get_vec_bool();
        // convert vec_bool to vector of 0 and 1
        let mut vec_u8: Vec<u8> = Vec::new();
        for bit in vec_bool {
            if *bit {
                vec_u8.push(1);
            } else {
                vec_u8.push(0);
            }
        }
        println!("{:?}", vec_u8);

    }

    #[test]
    fn test_write_bits_5() {
        let file_name = "../cowsay.txt-from-inflate";
        let file_path = Path::new(file_name);
        let data = read_file_to_byte_vector(file_path).unwrap();

        let mut bit_writer = BitWriter::new();
        bit_writer.write_bitstream_fixed_huffman(&data).unwrap();
        let buffer = bit_writer.get_buffer();
        println!("{:?}", buffer);
        let vec_bool = bit_writer.get_vec_bool();
        // convert vec_bool to vector of 0 and 1
        let mut vec_u8: Vec<u8> = Vec::new();
        for bit in vec_bool {
            if *bit {
                vec_u8.push(1);
            } else {
                vec_u8.push(0);
            }
        }
        println!("{:?}", vec_u8);

    }

    #[test]
    fn test_write_bits_6() {
        let file_name = "../testdata/inflate/fixed-distances-stress-expected";
        let file_path = Path::new(file_name);
        let data = read_file_to_byte_vector(file_path).unwrap();

        let mut bit_writer = BitWriter::new();
        bit_writer.write_bitstream_fixed_huffman(&data).unwrap();
        let buffer = bit_writer.get_buffer();
        println!("{:?}", buffer);
        let vec_bool = bit_writer.get_vec_bool();
        // convert vec_bool to vector of 0 and 1
        let mut vec_u8: Vec<u8> = Vec::new();
        for bit in vec_bool {
            if *bit {
                vec_u8.push(1);
            } else {
                vec_u8.push(0);
            }
        }
        println!("{:?}", vec_u8);

    }

    #[test]
    fn test_write_bits_7() {
        let file_name = "../testdata/inflate/fixed-lengths-stress-expected";
        let file_path = Path::new(file_name);
        let data = read_file_to_byte_vector(file_path).unwrap();

        let mut bit_writer = BitWriter::new();
        bit_writer.write_bitstream_fixed_huffman(&data).unwrap();
        let buffer = bit_writer.get_buffer();
        println!("{:?}", buffer);
        let vec_bool = bit_writer.get_vec_bool();
        // convert vec_bool to vector of 0 and 1
        let mut vec_u8: Vec<u8> = Vec::new();
        let mut skip_count = 0;
        let mut count = 0;
        let mut byte_num = 0;
        let mut to_skip = 3;

        // for bit in vec_bool {
        //     if byte_num == 0 && skip_count < to_skip {
        //         skip_count += 1;
        //         continue;
        //     }
        //     if count % 8 == 0 {
        //         byte_num += 1;
        //         println!(" ");
        //     }
        //     if *bit {
        //         print!("1");
        //     } else {
        //         print!("0");
        //     }
        //     count += 1;
        // }
        // println!("{:?}", vec_u8);

    }


    #[test]
    fn test_convert_distance_to_fixed_huffman_code() {
        let (code_value, extra_bits, extra_value) = convert_distance_to_fixed_huffman_code(1);
        println!("{}, {}, {}", code_value, extra_bits, extra_value);
        let (code_value, extra_bits, extra_value) = convert_distance_to_fixed_huffman_code(7);
        println!("{}, {}, {}", code_value, extra_bits, extra_value);
        let (code_value, extra_bits, extra_value) = convert_distance_to_fixed_huffman_code(16384);
        println!("{}, {}, {}", code_value, extra_bits, extra_value);
        let (code_value, extra_bits, extra_value) = convert_distance_to_fixed_huffman_code(23);
        println!("{}, {}, {}", code_value, extra_bits, extra_value);
        let (code_value, extra_bits, extra_value) = convert_distance_to_fixed_huffman_code(17);
        println!("{}, {}, {}", code_value, extra_bits, extra_value);
        
    }


    #[test]
    fn test_convert_length_to_fixed_huffman_code() {
        let (code_value, extra_bits, extra_value) = convert_length_to_fixed_huffman_code(3);
        println!("{}, {}, {}", code_value, extra_bits, extra_value);
        let (code_value, extra_bits, extra_value) = convert_length_to_fixed_huffman_code(10);
        println!("{}, {}, {}", code_value, extra_bits, extra_value);
        let (code_value, extra_bits, extra_value) = convert_length_to_fixed_huffman_code(156);
        println!("{}, {}, {}", code_value, extra_bits, extra_value);
        let real_huffman = convert_length_to_fixed_huffman_code_step_2(code_value);
        println!("{}", real_huffman);

    }

    #[test]
    fn test_convert_to_fixed_huffman_code_2() {
        let real_value = 'A' as u8;
        let fixed_huffman_code = convert_to_fixed_huffman_code(real_value);
        println!("{}", fixed_huffman_code);
    }
}                    