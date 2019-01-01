// Commonly used parsing helpers.

use std::iter::FlatMap;

// Iterates through bits, most significant first.
pub struct BitsIterMS {
    bit: i8,
    byte: u8,
}
impl BitsIterMS {
    pub fn new(byte: &u8) -> BitsIterMS {
        BitsIterMS { bit: 7, byte: *byte }
    }
}
impl Iterator for BitsIterMS {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        let this_bit = self.bit;
        if this_bit >= 0 {
            self.bit -= 1;
            Some((self.byte >> this_bit) & 1)
        } else {
            None
        }
    }
}
