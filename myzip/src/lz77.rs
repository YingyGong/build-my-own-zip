use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::path::Path;
use crate::bitwriter::BitWriter;

pub struct LZ77 {
    window_size: usize,
    lookahead_size: usize,
    hash_table: HashMap<Vec<u8>, Vec<usize>>,
}

impl LZ77 {
    pub fn new(window_size: usize, lookahead_size: usize) -> Self {
        LZ77 {
            window_size,
            lookahead_size,
            hash_table: HashMap::new(),
        }
    }

    pub fn add_to_hash(&mut self, key: &[u8], position: usize) {
        let key_vec = key.to_vec(); 
        self.hash_table.entry(key_vec).or_default().push(position);
    }

    pub fn update_hash(&mut self, cur_position: usize) {
        let window_start = cur_position.saturating_sub(self.window_size); // use saturating_sub to avoid underflow

        // delete those indexes that are no longer in the window
        for positions in self.hash_table.values_mut() {
            positions.retain(|&pos| pos >= window_start);
        }

        self.hash_table.retain(|_, v| !v.is_empty());
    }
}

pub struct OutputBuffer {
    buffer: Vec<u8>,
}

impl OutputBuffer {
    pub fn new() -> Self {
        OutputBuffer { buffer: Vec::new() }
    }

    pub fn append_literal(&mut self, literal: u8) {
        self.buffer.push(literal);
    }

    pub fn append_pair(&mut self, length: usize, distance: usize) {
        let formatted_pair = format!("<{},{}>", length, distance);
        self.buffer.extend_from_slice(formatted_pair.as_bytes());
    }

    pub fn write_to_file(&self, filename: &Path) -> io::Result<()> {
        let mut file = BufWriter::new(File::create(filename)?);
        file.write_all(&self.buffer)
    }
}

impl LZ77 {
    pub fn compress(&mut self, input: &[u8], bitwriter: &mut BitWriter) {
        let mut i = 0;
        while i < input.len() {
            if i + 3 > input.len() {
                bitwriter.write_single_literal(input[i]);
                i += 1;
                continue;
            }            

            let key = &input[i..i + 3];
            if i > self.window_size {
                self.update_hash(i);
            }
            // look for the key in the hash table
            if let Some(positions) = self.hash_table.get(key) {
                let mut match_length = 3;
                let mut match_distance = 0;
                for &pos in positions {
                    let mut j = 3;
                    while i + j < input.len() && j < self.lookahead_size {
                        if input[i + j] != input[pos + j] {
                            break;
                        }
                        j += 1;
                    }
                    if j >= match_length {
                        match_length = j;
                        match_distance = i - pos;
                    }
                }


                if match_length > 0 {
                    println!("match length: {}, match distance: {}", match_length, match_distance);
                    bitwriter.write_length(match_length as u16);
                    bitwriter.write_distance(match_distance as u16);
                    for k in i..i + match_length {
                        if k + 3 > input.len() {
                            break;
                        }
                        let key = &input[k..k + 3];
                        self.add_to_hash(key, k);
                    }

                    i += match_length;
                } else { // theoretically this should never happen
                    bitwriter.write_single_literal(input[i]);
                    i += 1;
                }
            } else {
                bitwriter.write_single_literal(input[i]);
                self.add_to_hash(key, i);
                i += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_lz77() {
    //     let mut compressor = LZ77::new(32768, 258);
    //     let mut output_buffer = OutputBuffer::new();

    //     let input = b"abcabcaaaaabcabcaaooooooabcabaca";
    //     compressor.compress(input, &mut output_buffer);
    //     let output_str = std::str::from_utf8(&output_buffer.buffer).unwrap();
    //     println!("{}", output_str);

    //     // let expected_output = b"abracad<5,4>";
    //     // assert_eq!(output_buffer.buffer, expected_output);
    // }
}