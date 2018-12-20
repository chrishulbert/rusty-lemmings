// This is for decompressing lemmings DAT files:
// https://www.camanis.net/lemmings/files/docs/lemmings_dat_file_format.txt

use std::io;
use std::io::Error;
use std::io::ErrorKind;

// Safe byte retrieval.
fn byte_at(buffer: &[u8], index: usize) -> io::Result<u8> {
    if index < buffer.len() {
        Ok(buffer[index])
    } else {
        Err(Error::new(ErrorKind::UnexpectedEof, "End of buffer"))
    }
}

// Iterator that returns the first N bits of a byte.
struct NBitsIter {
    bit: u8,
    byte: u8,
    bits: u8,
}
impl NBitsIter {
    fn new(byte: u8, bits: u8) -> NBitsIter {
        NBitsIter { bit: 0, byte: byte, bits: bits }
    }
}
impl Iterator for NBitsIter {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        let this_bit = self.bit;
        if this_bit < self.bits {
            self.bit += 1;
            Some((self.byte >> this_bit) & 1)
        } else {
            None
        }
    }
}

// if offset=0, returns the end byte.
// if offset=1, returns the one just before the end, and so on.
fn from_end(vec: &Vec<u8>, offset: isize) -> io::Result<u8> {
    let index: isize = vec.len() as isize - 1 - offset;
    if index < 0 {
        Err(Error::new(ErrorKind::Other, "Bad offset"))
    } else {
        Ok(vec[index as usize])
    }
}

// Exposes the 'next' as a result so you can use '?'.
fn read_bit(bits: &mut Iterator<Item = u8>) -> io::Result<u8> {
    match bits.next() {
        Some(t) => Ok(t),
        None => Err(Error::new(ErrorKind::UnexpectedEof, "No bits remaining")),
    }
}

// Converts the bit to a bool for typesafe matching without default clauses.
fn read_bool(bits: &mut Iterator<Item = u8>) -> io::Result<bool> {
    Ok(read_bit(bits)? == 1)
}

fn read_byte(bits: &mut Iterator<Item = u8>) -> io::Result<u8> {
    let b1 = read_bit(bits)?;
    let b2 = read_bit(bits)?;
    let b3 = read_bit(bits)?;
    let b4 = read_bit(bits)?;
    let b5 = read_bit(bits)?;
    let b6 = read_bit(bits)?;
    let b7 = read_bit(bits)?;
    let b8 = read_bit(bits)?;
    Ok((b1<<7) + (b2<<6) + (b3<<5) + (b4<<4) + (b5<<3) + (b6<<2) + (b7<<1) + b8)
}

/// Recursively decompresses the sections from a file.
fn decompress_recursively(compressed: &[u8], sections: Vec<Vec<u8>>) -> io::Result<Vec<Vec<u8>>> {
    let num_bits_in_first_byte = byte_at(compressed, 0)?;
    let checksum = byte_at(compressed, 1)?;
    let decompressed_data_size: u16 = ((byte_at(compressed, 4)? as u16) << 8) + (byte_at(compressed, 5)? as u16);
    let compressed_data_end: usize = ((byte_at(compressed, 8)? as usize) << 8) + (byte_at(compressed, 9)? as usize);
    if compressed_data_end > compressed.len() { 
        return Err(Error::new(ErrorKind::UnexpectedEof, "Past EOF: compressed_data_end"));
    }

    // Validate the checksum.
    let mut calculated_checksum: u8 = 0;
    for byte in compressed[10..compressed_data_end].iter() {
        calculated_checksum ^= byte;
    }
    if calculated_checksum != checksum {
        return Err(Error::new(ErrorKind::Other, "Checksum"))
    }

    let mut bits = compressed[10..compressed_data_end].iter().rev().enumerate().flat_map(|(i, val)| {
        let num_bits: u8 = if i==0 { num_bits_in_first_byte } else { 8 };
        return NBitsIter::new(*val, num_bits);
    });
    let mut decompressed: Vec<u8> = Vec::new();
    while decompressed.len() < decompressed_data_size as usize {
        match read_bool(&mut bits)? {
            false => {
                match read_bool(&mut bits)? {
                    false => { // 1: some raw bytes
                        let n1 = read_bit(&mut bits)?;
                        let n2 = read_bit(&mut bits)?;
                        let n3 = read_bit(&mut bits)?;
                        let n = (n1 << 2) + (n2 << 1) + n3 + 1;
                        for _ in 0..n {
                            decompressed.push(read_byte(&mut bits)?);
                        }
                    },
                    true => { // 2: Reuse 2 bytes.
                        let offset = read_byte(&mut bits)? as isize;
                        for _ in 0..2 {
                            let b = from_end(&decompressed, offset)?;
                            decompressed.push(b);
                        }
                    }
                }
            },
            true => {
                let b = read_bool(&mut bits)?;
                let c = read_bool(&mut bits)?;
                match (b, c) {
                    (false, false) => { // 3: reuse 3 bytes.
                        let m1 = read_bit(&mut bits)?;
                        let m2 = read_byte(&mut bits)?;
                        let offset: isize = ((m1 as isize) << 8) + (m2 as isize);
                        for _ in 0..3 {
                            let b = from_end(&decompressed, offset)?;
                            decompressed.push(b);
                        }
                    },
                    (false, true) => { // 4: reuse 4 bytes.
                        let m1 = read_bit(&mut bits)?;
                        let m2 = read_bit(&mut bits)?;
                        let m3 = read_byte(&mut bits)?;
                        let offset: isize = ((m1 as isize) << 9) + ((m2 as isize) << 8) + (m3 as isize);
                        for _ in 0..4 {
                            let b = from_end(&decompressed, offset)?;
                            decompressed.push(b);
                        }
                    },
                    (true, false) => { // 5: reuse N bytes.
                        let n = read_byte(&mut bits)?;
                        let length: u16 = n as u16 + 1;
                        let m1 = read_bit(&mut bits)?;
                        let m2 = read_bit(&mut bits)?;
                        let m3 = read_bit(&mut bits)?;
                        let m4 = read_bit(&mut bits)?;
                        let m5 = read_byte(&mut bits)?;
                        let offset: isize = ((m1 as isize) << 11) + ((m2 as isize) << 10) + ((m3 as isize) << 9) + ((m4 as isize) << 8) + (m5 as isize);
                        for _ in 0..length {
                            let b = from_end(&decompressed, offset)?;
                            decompressed.push(b);
                        }
                    },
                    (true, true) => { // 6: many raw bytes.
                        let n = read_byte(&mut bits)?;
                        let length: u16 = n as u16 + 9;
                        for _ in 0..length {
                            decompressed.push(read_byte(&mut bits)?);
                        }
                    }
                }
            }
        }
    }
    decompressed.reverse();

    let remaining_compressed_data = &compressed[compressed_data_end..];
    let mut all_sections = sections;
    all_sections.push(decompressed);
    if remaining_compressed_data.len() == 0 {
        // No more data, can finish recursing now.
        return Ok(all_sections);
    } else {
        // Recurse for further sections.
        return decompress_recursively(remaining_compressed_data, all_sections);
    }
}

/// Decompresses all the sections from a compressed dat file.
/// Returns a vec of sections. Each section is a vec of its data.
pub fn decompress(compressed: &[u8]) -> io::Result<Vec<Vec<u8>>> {
    decompress_recursively(&compressed, Vec::new())
}
