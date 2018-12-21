// This extracts sprites from data.

struct BitsIter {
    bit: i8,
    byte: u8,
}
impl BitsIter {
    fn new(byte: u8) -> BitsIter {
        BitsIter { bit: 7, byte: byte }
    }
}
impl Iterator for BitsIter {
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

// Creates a bit iterator from [u8].
macro_rules! iterate_bits { ($data:expr) => { $data.iter().flat_map(|byte| { return BitsIter::new(*byte); }); } }

// Extract a single sprite.
// Sprites are stored as 4 planes, eg all the 1 bits, then all the 2 bits, then so on. It seems they did this so
// they could reuse one of the bits as the mask plane. But it means parsing is weird.
// Palette entries are 0xRRGGBBAA
pub fn extract(data: &[u8], width: usize, height: usize, image_loc: u16, mask_loc: u16, palette: &[u32; 16]) -> Vec<u32> {
    let image_data: &[u8] = &data[image_loc as usize..];
    let mask_data: &[u8] = &data[mask_loc as usize..];
    let pixels: usize = width * height;
    let mut image_iter_0 = iterate_bits!(image_data);
    let mut image_iter_1 = iterate_bits!(image_data).skip(pixels);
    let mut image_iter_2 = iterate_bits!(image_data).skip(pixels * 2);
    let mut image_iter_3 = iterate_bits!(image_data).skip(pixels * 3);
    let mut mask_iter = iterate_bits!(mask_data);
    let mut sprite: Vec<u32> = Vec::new();
    for _ in 0..pixels {
        let colour_index =
            image_iter_0.next().unwrap() +
            (image_iter_1.next().unwrap() << 1) +
            (image_iter_2.next().unwrap() << 2) +
            (image_iter_3.next().unwrap() << 3);
        let colour: u32 = palette[colour_index as usize];
        let alpha: u32 = if mask_iter.next().unwrap() == 0 { 0 } else { 0xff000000 };
        let masked_colour = (colour & 0xffffff) + alpha;
        sprite.push(masked_colour);
    }
    return sprite;
}
