use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::path::Path;

pub struct LZ77 {
    window_size: usize,
    lookahead_size: usize,
    hash_table: HashMap<String, Vec<usize>>,  // stores positions of substrings
}

impl LZ77 {
    pub fn new(window_size: usize, lookahead_size: usize) -> Self {
        LZ77 {
            window_size,
            lookahead_size,
            hash_table: HashMap::new(),
        }
    }

    pub fn add_to_hash(&mut self, key: &str, position: usize) {
        self.hash_table.entry(key.to_string()).or_default().push(position);
    }

    // delete keys that are no longer in the window
    pub fn update_hash(&mut self, cur_position: usize) {
        let window_start = cur_position - self.window_size;
        // last ele in the latest appearance, delete those keys that are no longer in the window
        self.hash_table.retain(|_, positions| {
            if let Some(last_position) = positions.last() {
                *last_position >= window_start
            } else {
                false
            }
        });
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
    pub fn compress(&mut self, input: &[u8], output: &mut OutputBuffer) {
        let mut i = 0;
        while i < input.len() {
            if i + 3 > input.len() {
                output.append_literal(input[i]);
                i += 1;
                continue;
            }            

            let key = &input[i..i + 3];
            let key_str = std::str::from_utf8(key).unwrap();
            if i > self.window_size {
                self.update_hash(i);
            }
            // look for the key in the hash table
            if let Some(positions) = self.hash_table.get(key_str) {
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
                    output.append_pair(match_length, match_distance);
                    for k in i..i + match_length {
                        if k + 3 > input.len() {
                            break;
                        }
                        let key_str = std::str::from_utf8(&input[k..k + 3]).unwrap();
                        self.add_to_hash(key_str, k);
                    }

                    i += match_length;
                } else { // theoretically this should never happen
                    output.append_literal(input[i]);
                    i += 1;
                }
            } else {
                output.append_literal(input[i]);
                self.add_to_hash(key_str, i);
                i += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lz77() {
        let mut compressor = LZ77::new(32768, 258);
        let mut output_buffer = OutputBuffer::new();

        let input = b"abcabcaaaaabcabcaaooooooabcabaca";
        compressor.compress(input, &mut output_buffer);
        let output_str = std::str::from_utf8(&output_buffer.buffer).unwrap();
        println!("{}", output_str);

        // let expected_output = b"abracad<5,4>";
        // assert_eq!(output_buffer.buffer, expected_output);
    }
}