use std::collections::HashMap;
use std::hash::Hash;
use std::{io, vec, mem};
use std::{env, fs};
use std::fs::File;
use std::io::{Read, Write, Cursor, Seek, SeekFrom};
use std::path::Path;

fn read_file_to_byte_vector(file_path: &Path) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}


pub fn check_valid_conversion(count: u8, value_from_binary: u16) -> u16 {
    if (count == 7 && value_from_binary <= 23) {
        return value_from_binary + 256
    } else if (count == 8) {
        if (value_from_binary >= 48 && value_from_binary <= 191) {
            return value_from_binary - 48
        }
        else if (value_from_binary >= 192 && value_from_binary <= 199) {
            return value_from_binary + 88
        }
        else {
            return 300
        }
    } else if (count == 9 && value_from_binary >= 400 && value_from_binary <= 511)  {
        return  value_from_binary - 256
    } else {
        return 300
    }
    
}

pub fn get_mapping_from_canonical_huffman_lengths(list_lengths: Vec<usize>, alphabets: Vec<usize>) -> Vec<HashMap<u16, u16>>{   
    if list_lengths.len() == 0 {
        return Vec::new();
    }

    let max_length = list_lengths.iter().max().unwrap();
    let num_alphabets: usize = list_lengths.len();
    let mut map: Vec<HashMap<u16, u16>> = vec![HashMap::new(); *max_length + 1];
    let mut bl_count = vec![0u16; *max_length + 1];

    // count the number of codes for each code length
    for length in list_lengths.iter() {
        bl_count[*length] += 1;
    }
    // println!("bl count is {:?}", bl_count);

    // find the numerical value of the smallest code for each code length
    let mut next_codes: Vec<u16> = vec![0; *max_length + 1];
    let mut code = 0;
    for bits in 1..=*max_length {
        code = (code + bl_count[bits - 1]) << 1;
        next_codes[bits] = code;
    }
    // println!("next codes is {:?}", next_codes);

    for i in 0..num_alphabets {
        if list_lengths[i] == 0 {
            continue;
        }
        let length = list_lengths[i];
        let code: u16 = next_codes[length];
        next_codes[length] += 1;
        map[length].insert(code, alphabets[i] as u16);
    }

    map
}

fn delete_zero_from_lengths_and_alphabets(list_lengths: Vec<usize>, alphabets: Vec<usize>) -> (Vec<usize>, Vec<usize>) {
    let mut new_list_lengths: Vec<usize> = Vec::new();
    let mut new_alphabets: Vec<usize> = Vec::new();
    for i in 0..list_lengths.len() {
        if list_lengths[i] != 0 {
            new_list_lengths.push(list_lengths[i]);
            new_alphabets.push(alphabets[i]);
        }
    }
    (new_list_lengths, new_alphabets)
}

fn generate_no_zero_lengths_and_alphabets(list_lengths: Vec<usize>) -> (Vec<usize>, Vec<usize>) {
    let mut new_list_lengths: Vec<usize> = Vec::new();
    let mut new_alphabets: Vec<usize> = Vec::new();
    for i in 0..list_lengths.len() {
        if list_lengths[i] != 0 {
            new_list_lengths.push(list_lengths[i]);
            new_alphabets.push(i);
        }
    }
    (new_list_lengths, new_alphabets)
}

pub struct BitReader<'a> {
    data: &'a [u8],
    position: usize,
    vec_bool: Vec<bool>,
    resulted_bytes: Vec<u8>,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let mut bitreader = BitReader{ data, position: 0, vec_bool: Vec::new(), resulted_bytes: Vec::new()};
        bitreader.get_vec_bool();
        bitreader
    }

    pub fn read_bits(&mut self, count: u8, reverse: bool) -> io::Result<u16> {
        // access vec_bool to get bit
        let mut result: u16 = 0;
        let count = count as usize;
        for i in 0..count {
            let cur_position = self.position + i;
            let cur_bit = self.vec_bool[cur_position] as u16;
            if reverse {
                result += cur_bit << i; // shift right
            } else {
            result += cur_bit << (count - i - 1); // shift left
            }
        }
        self.position += count;
        Ok(result)
    }


    pub fn match_fixed_huffman(&mut self) -> io::Result<u16> {
        
        let mut result: u16 = 0;
        let counts: Vec<u8> = vec![7, 8, 9];
        for count in counts {
            result = self.read_bits(count, false)?;
            let result_conversed = check_valid_conversion(count, result);
            if result_conversed == 300 { // magic number for: not matched, try next
                self.position -= count as usize;
            } 
            else {
                result = result_conversed;
                break;
            }
        }

        Ok(result)
    }

    // for debug
    fn print_bitstream_desired_range(&mut self, start: usize, end: usize) {
        for i in start..end {
            print!("{}", if self.vec_bool[i] {1} else {0});
        }
        println!();
    }

    // to put a static table outside ?
    fn read_length(&mut self, code: u16) -> u16 {
        assert!(code > 256);
        let mut designation: u16 = 0;
        if code < 265 {
            return code - 254
        }
        else if code < 269 {
            designation = self.read_bits(1, true).unwrap();
            return 2 * (code - 265) + 11 + designation
        }
        else if code < 273 {
            designation = self.read_bits(2, true).unwrap();
            return 4 * (code - 269) + 19 + designation
        }
        else if code < 277 {
            designation = self.read_bits(3, true).unwrap();
            return 8 * (code - 273) + 35 + designation
        }
        else if code < 281 {
            designation = self.read_bits(4, true).unwrap();
            return 16 * (code - 277) + 67 + designation
        }
        else if code < 285 {
            designation = self.read_bits(5, true).unwrap();
            return 32 * (code - 281) + 131 + designation
        }
        else {
            assert!(code == 285);
            return 258
        }
    }

    fn read_distance(&mut self, length_code: u16) -> u16 {
        if length_code < 4 {
            return length_code + 1
        }
        let extra_bits =  ((length_code - 2) / 2 ) as u32 ;
        let to_add = self.read_bits(extra_bits as u8, true).unwrap();
        if length_code % 2 == 0 {
            return 2u16.pow(extra_bits + 1) + 1 + to_add
        } else {
            return 2u16.pow(extra_bits + 1) + 1 + to_add + 2u16.pow(extra_bits)
        }
    }

    // read one block 
    fn read_one_block(&mut self) -> io::Result<()> {
        let bfinal = self.read_bits(1, false)?;
        let fill_the_end_to_multiple_8 = bfinal == 1;

        let btype = self.read_bits(2, true)?;
        if btype == 1 { // fixed huffman
            self.read_fixed_block(fill_the_end_to_multiple_8)?
        } else {
            self.read_dynamic_block(fill_the_end_to_multiple_8)?
        }

        Ok(())
    }

    fn read_fixed_block(&mut self, bfinal: bool) -> io::Result<()> {
        let mut cur_len: usize = 0;
        while self.position < self.data.len() * 8{
            let next_code = self.match_fixed_huffman()?;

            assert!(next_code <= 285);
            if next_code == 256 { // EOB
                break;
            }
            else if next_code > 256 {
                let len = self.read_length(next_code) as usize;
                let distance_code = self.read_bits(5, false).unwrap();
                let distance = self.read_distance(distance_code) as usize;

        
                let start = self.resulted_bytes.len() - distance;
                if len <= distance {
                    let repeat_sequence = self.resulted_bytes[start..(start + len)].to_vec();
                    self.resulted_bytes.extend(&repeat_sequence);
                } else {
                    for i in 0..len {
                        let index = i + start;
                        self.resulted_bytes.push(self.resulted_bytes[index]);
                    }
                }
                cur_len += len;
            }
            else  
            {
                self.resulted_bytes.push(next_code as u8);
                cur_len += 1;
            }
        }
        if ! bfinal {
            self.read_one_block()
        } else {
            Ok(()) // last block
        }
    }

    fn read_dynamic_block(&mut self, bfinal: bool) -> io::Result<()> {
        let (hlit, hdist, hclen) = self.parse_dynamic_header()?;
        let (key_vector, value_vector) = self.read_hclen(hclen)?;
        let hclen_map = get_mapping_from_canonical_huffman_lengths(value_vector, key_vector);
        let hlit_map = self.get_hlit_or_hdist_map(hlit, &hclen_map)?;
        let hdist_map = self.get_hlit_or_hdist_map(hdist, &hclen_map)?;

        let mut cur_len: usize = 0;
        while self.position < self.data.len() * 8{
            // println!("cur_len is {}", cur_len);
            let next_code = self.decode_one_dynamic_huffman(&hlit_map)?;

            assert!(next_code <= 285);
            if next_code == 256 { // EOB
                break;
            }
            else if next_code > 256 {
                let len = self.read_length(next_code) as usize;
                // println!("len is {}", len);

                let distance_code = self.decode_one_dynamic_huffman(&hdist_map)?;

                let distance = self.read_distance(distance_code) as usize;

                // assert!(cur_len >= distance); 

        
                let start = self.resulted_bytes.len() - distance;
                if len <= distance {
                    let repeat_sequence = self.resulted_bytes[start..(start + len)].to_vec();
                    self.resulted_bytes.extend(&repeat_sequence);
                } else {
                    for i in 0..len {
                        let index = i + start;
                        self.resulted_bytes.push(self.resulted_bytes[index]);
                    }
                }
                cur_len += len;
            }
            else  
            {
                self.resulted_bytes.push(next_code as u8);
                cur_len += 1;
            }
        }
        if ! bfinal {
            self.read_one_block()
        } else {
            Ok(()) // last block
        }
    }

    pub fn read_bitstream_blocks(&mut self) -> io::Result<Vec<u8>> {
        self.read_one_block();
        // replace self.resulted_bytes with a new empty vector, return the original vector
        Ok(mem::take(&mut self.resulted_bytes)) 
    }
    
    // deprecated
    pub fn read_bitstream_fixed_huffman(&mut self) -> io::Result<Vec<u8>> {
        let mut result: Vec<u8> = Vec::new();
        let mut cur_len: usize = 0;

        while self.position < self.data.len() * 8{
            // println!("{}", self.position);
            let next_code = self.match_fixed_huffman()?;

            assert!(next_code <= 285);
            if next_code == 256 { // EOB
                break;
            }
            else if next_code > 256 {
                let len = self.read_length(next_code) as usize;
                let distance_code = self.read_bits(5, false).unwrap();
                let distance = self.read_distance(distance_code) as usize;

                // if cur_len < distance {
                //     println!("self.position: {}", self.position);
                //     println!("len {} , cur_len: {}, distance: {}",len, cur_len, distance);
                // }
                assert!(cur_len >= distance); 
                // if debug {
                //     println!("len {} , cur_len: {}, distance: {}",len, cur_len, distance);
                //     debug = false;
                // }
        
                let start = cur_len - distance;
                if len <= distance {
                    let repeat_sequence = result[start..(start + len)].to_vec();
                    result.extend(&repeat_sequence);
                } else {
                    // println!("len: {}, distance: {}", len, distance);
                    // let repeat_time = len / distance;
                    // let repeat_sequence = result[(cur_len - distance)..(cur_len)].to_vec();
                    // for _ in 0..repeat_time {
                    //     result.extend(repeat_sequence.clone());
                    // }
                    for i in 0..len {
                        let index = i % distance + start;
                        result.push(result[index]);
                    }
                }
                cur_len += len;
            }
            else  
            {
                result.push(next_code as u8);
                cur_len += 1;
            }
        }
        Ok(result)
    }

    // read the whole data
    pub fn get_vec_bool(&mut self) {
        let size: usize = self.data.len() * 8;
        let mut result: Vec<bool> = Vec::with_capacity(size);
        for i in 0..size {
            let byte_index = i / 8;
            let bit_index = i % 8;
            let bit = (self.data[byte_index] >> bit_index) & 1;
            result.push(bit != 0);
        }
        self.vec_bool = result;
    }

    // dynamic huffman code

    fn parse_dynamic_header(&mut self) -> io::Result<(usize, usize, usize)> {
        let hlit = self.read_bits(5, true)? as usize + 257;  // number of literal/length codes
        let hdist = self.read_bits(5, true)? as usize + 1;   // number of distance codes
        let hclen = self.read_bits(4, true)? as usize + 4;   // number of code length codes
    
        Ok((hlit, hdist, hclen))
    }

    fn read_hclen(&mut self, hclen: usize) -> io::Result<(Vec<usize>, Vec<usize>)> {
        let mut code_length_map = HashMap::new();
        let order = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

        for i in 0..hclen {
            let code = self.read_bits(3, true)?;
            if code == 0 {
                continue;
            }
            code_length_map.insert(order[i], code);
        }
        
        // key_vector is alphabet, value_vector is code length
        let mut key_vector: Vec<usize> = Vec::new();
        let mut value_vector: Vec<usize> = Vec::new();
        for i in 0..19 {
            if let Some(&value) = code_length_map.get(&i) {
                if value != 0 {
                    key_vector.push(i);
                    value_vector.push(value as usize);
                }
            }
        }
        Ok((key_vector, value_vector))

    }

    fn decode_one_dynamic_huffman(&mut self, map: &Vec<HashMap<u16, u16>>) -> io::Result<u16> {
        let mut code: u16 = 0;
        let mut count: u8 = 1;
        loop {
            let next_huffman_code = self.read_bits(count, false)?;
            // try to find the code in the map
            if let Some(&value) = map[count as usize].get(&next_huffman_code) {
                code = value;
                break;
            }
            self.position -= count as usize;
            count += 1;
        }
        // println!("code is {} at length {}", code, count);
        Ok(code)
    }

    // fn decode_hclen_huffman(&mut self, hclen_map: &Vec<HashMap<u16, u16>>) -> io::Result<Vec<usize>> {
    //     let mut code_lengths: Vec<usize> = vec![0; 19];
    //     for i in 0..19 {
    //         let code = self.decode_one_dynamic_huffman()?;
    //         code_lengths[i] = code as usize;
    //     }
    //     Ok(code_lengths)
    // }

    fn read_and_decode_dynamic_huffman(&mut self, num_huffman_codes: usize, map: &Vec<HashMap<u16, u16>>) -> io::Result<(Vec<usize>, Vec<usize>)> {
        let mut code_lengths: Vec<usize> = Vec::new();
        let mut alphabets: Vec<usize> = Vec::new();
        let mut cur_alpha = 0;
        let mut i = 0;
        let mut last_code = 0;
        while i < num_huffman_codes {
            let code = self.decode_one_dynamic_huffman(map)?;
            if code < 16 {
                if code != 0 {
                    code_lengths.push(code as usize);
                    alphabets.push(cur_alpha);
                }
                last_code = code;
                cur_alpha += 1;
                i += 1;
                
            } else if code == 16 {
                let repeat = self.read_bits(2, true)? as usize;
                if last_code != 0 {
                    for _ in 0..(repeat + 3) {
                        code_lengths.push(last_code as usize);
                        alphabets.push(cur_alpha);
                        cur_alpha += 1;
                    }
                }
                i += repeat + 3;
            } else if code == 17 {
                let repeat = self.read_bits(3, true)? as usize;
                i += repeat + 3;
                cur_alpha += repeat + 3;
                last_code = 0;
            } else if code == 18 {
                let repeat = self.read_bits(7, true)? as usize;
                i += repeat + 11;
                cur_alpha += repeat + 11;
                last_code = 0;
            }
        }

        // println!("list {:?}", code_lengths);
        // println!("alphabets {:?}", alphabets);
        Ok((code_lengths, alphabets))
    }


    fn get_hlit_or_hdist_map(&mut self, hlit: usize, hclen_map: &Vec<HashMap<u16, u16>>) -> io::Result<Vec<HashMap<u16, u16>>> {
        let (list_lengths, alphabets) = self.read_and_decode_dynamic_huffman(hlit, hclen_map)?;
        let map = get_mapping_from_canonical_huffman_lengths(list_lengths, alphabets);
        Ok(map)
    }

    // fn get_hdist_map(&mut self, hdist: usize, hclen_map: &Vec<HashMap<u16, u16>>) -> io::Result<Vec<HashMap<u16, u16>>> {
    //     let code_lengths = self.read_and_decode_dynamic_huffman(hdist, hclen_map)?;
    //     let map = get_mapping_from_canonical_huffman_lengths(code_lengths);
    //     Ok(map)
    // }





}


#[cfg(test)]
mod tests {
    use super::*;

    fn setup_bitreader<'a>(data: &'a [u8]) -> BitReader<'a> {
        BitReader::new(data)
    }

    #[test]
    fn test_read_bits_1() {
        match env::current_dir() {
            Ok(dir) => println!("Current directory: {}", dir.display()),
            Err(e) => println!("Error getting current directory: {}", e),
        }
        let file_name = "../testdata/inflate/fixed-huffman-literals.deflate";
        let file_path = Path::new(file_name);
        let data = read_file_to_byte_vector(file_path).unwrap();
        println!("{:?}", data);

        let mut bit_reader = BitReader::new(&data);
        // bit_reader.print_bitstream();
        bit_reader.read_bits(1, false);
        println!("{}", bit_reader.read_bits(2, false).unwrap());
        println!("{}", bit_reader.read_bits(8, false).unwrap());

    }

    #[test]
    fn test_read_bits_2() {
        let file_name = "../testdata/inflate/fixed-huffman-overlapping-run0.deflate";
        let file_path = Path::new(file_name);
        let data = read_file_to_byte_vector(file_path).unwrap();

        let mut bit_reader = BitReader::new(&data);
        bit_reader.read_bits(3, false);
        let decoded_results = bit_reader.read_bitstream_fixed_huffman().unwrap();
        println!("{:?}", decoded_results);
    }

    #[test]
    fn test_read_bits_3() {
        let file_name = "../testdata/inflate/fixed-lengths-stress.deflate";
        let file_path = Path::new(file_name);
        let data = read_file_to_byte_vector(file_path).unwrap();
        let mut bit_reader = BitReader::new(&data);
        bit_reader.read_bits(3, false);
        let decoded_results = bit_reader.read_bitstream_fixed_huffman().unwrap();
        println!("{:?}", decoded_results);
    }

    #[test]
    fn test_read_bits_4() {
        let file_name = "../testdata/inflate/dynamic-huffman-empty.deflate";
        let file_path = Path::new(file_name);
        let data = read_file_to_byte_vector(file_path).unwrap();
        let mut bit_reader = BitReader::new(&data);
        let decoded_results = bit_reader.read_bitstream_blocks().unwrap();
        println!("{:?}", decoded_results);
    }

    #[test]
    fn test_read_bits_5() {
        let file_name = "../testdata/inflate/dynamic-huffman-empty-no-distance-code.deflate";
        let file_path = Path::new(file_name);
        let data = read_file_to_byte_vector(file_path).unwrap();
        let mut bit_reader = BitReader::new(&data);
        let decoded_results = bit_reader.read_bitstream_blocks().unwrap();
        println!("{:?}", decoded_results);
    }

    #[test]
    fn test_read_bits_6() {
        let file_name = "../testdata/inflate/dynamic-huffman-one-distance-code.deflate";
        let file_path = Path::new(file_name);
        let data = read_file_to_byte_vector(file_path).unwrap();
        let mut bit_reader = BitReader::new(&data);
        let decoded_results = bit_reader.read_bitstream_blocks().unwrap();
        println!("decoded result is {:?}", decoded_results);
    }

    #[test]
    fn test_read_bits_7() {
        let file_name = "../bbrot.pgm.deflate";
        let file_path = Path::new(file_name);
        let data = read_file_to_byte_vector(file_path).unwrap();
        let mut bit_reader = BitReader::new(&data);
        let decoded_results = bit_reader.read_bitstream_blocks().unwrap();
        println!("decoded result is {:?}", decoded_results);
    }

    

    #[test]
    fn test_parse_dynamic_header() {
        let file_names = ["../testdata/inflate/dynamic-huffman-one-distance-code.deflate",
                          "../testdata/inflate/dynamic-huffman-empty-no-distance-code.deflate", 
                          "../testdata/inflate/dynamic-huffman-empty.deflate"];
        for file_name in file_names.iter() {
            let file_path = Path::new(file_name);
            let data = read_file_to_byte_vector(file_path).unwrap();
            let mut bit_reader = BitReader::new(&data);
            bit_reader.read_bits(3, false);
            let (hlit, hdist, hclen) = BitReader::parse_dynamic_header(&mut bit_reader).unwrap();
            println!("hlit: {}, hdist: {}, hclen: {}", hlit, hdist, hclen);
        }
    }

    #[test]
    fn test_get_mapping_from_canonical_huffman_lengths() {
        let list_lengths = vec![3, 3, 3, 3, 3, 2, 4, 4];
        let alphabets = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let result = get_mapping_from_canonical_huffman_lengths(list_lengths, alphabets);
        println!("{:?}", result);
    }

    // I used dynamic-huffman-one-distance-code as test case
    #[test]
    fn test_get_mapping_from_canonical_huffman_lengths_2() {
        let list_lengths = vec![2; 4];
        let alphabets = vec![0, 1, 2, 18];
        let result = get_mapping_from_canonical_huffman_lengths(list_lengths, alphabets);
        println!("{:?}", result);
        
        let list_lengths = vec![2, 1, 2];
        let alphabets = vec![1, 256, 257];
        let result = get_mapping_from_canonical_huffman_lengths(list_lengths, alphabets);
        println!("hlit map is {:?}", result);

        let list_lengths = vec![1];
        let alphabets = vec![0];
        let result = get_mapping_from_canonical_huffman_lengths(list_lengths, alphabets);
        println!("hlit map is {:?}", result);
    }

    #[test]
    fn test_dynamic_huffman_empty() {
        let list_lengths = vec![1, 1];
        let alphabets = vec![1,18];
        let map = get_mapping_from_canonical_huffman_lengths(list_lengths, alphabets);
        println!("{:?}", map);

        // let list_lengths = vec![1, 1];
        // let alphabets = vec![0, 256];
        // let result = get_mapping_from_canonical_huffman_lengths(list_lengths, alphabets);
        // println!("hlit map is {:?}", result);

        // let list_lengths = vec![1];
        // let alphabets = vec![0];
        // let result = get_mapping_from_canonical_huffman_lengths(list_lengths, alphabets);
        // println!("hlit map is {:?}", result);
    }
}