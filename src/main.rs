use std::fs;
use std::io;

mod decompressor;
mod ground;

struct BitsIter {
    bit: u8,
    byte: u8,
}
impl BitsIter {
    fn new(byte: u8) -> BitsIter {
        BitsIter { bit: 0, byte: byte }
    }
}
impl Iterator for BitsIter {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        let this_bit = self.bit;
        if this_bit < 8 {
            self.bit += 1;
            Some((self.byte >> this_bit) & 1)
        } else {
            None
        }
    }
}

macro_rules! iterate_bits { ($data:expr) => { $data.iter().flat_map(|byte| { return BitsIter::new(*byte); }); } }

// Sprites are stored as 4 planes, eg all the 1 bits, then all the 2 bits, then so on. It seems they did this so
// they could reuse one of the bits as the mask plane. But it means parsing is weird.
fn extract_sprite(data: &[u8], width: u8, height: u8, image_loc: u16, mask_loc: u16) -> Vec<u32> {
    let image_data: &[u8] = &data[image_loc as usize..];
    let mask_data: &[u8] = &data[mask_loc as usize..];
    let pixels: usize = (width as usize) * (height as usize);
    let mut image_iter_0 = iterate_bits!(image_data);
    let mut image_iter_1 = iterate_bits!(image_data).skip(pixels);
    let mut image_iter_2 = iterate_bits!(image_data).skip(pixels * 2);
    let mut image_iter_3 = iterate_bits!(image_data).skip(pixels * 3);
    let mut mask_iter = iterate_bits!(mask_data);
    for pixel in 0..pixels {
        let colour =
            image_iter_0.next().unwrap() +
            (image_iter_1.next().unwrap() << 1) +
            (image_iter_2.next().unwrap() << 2) +
            (image_iter_3.next().unwrap() << 3);
        let alpha = mask_iter.next().unwrap();
    }
    return Vec::new();
}

fn main() -> io::Result<()> {
    let raw: Vec<u8> = fs::read("data/VGAGR0.DAT")?;
    let data = decompressor::decompress(&raw)?;
    println!("Number of sections: {:?}", data.len());
    for s in 0..data.len() {
        println!("Section #{}: {} bytes", s, data[s].len());
    }


    let ground_file: Vec<u8> = fs::read("data/GROUND0O.DAT")?;
    let ground = ground::parse(&ground_file)?;
    println!("VGA: {:x}", ground.palettes.vga_custom[0]);


    for (i, terrain) in ground.terrain_info.iter().enumerate() {
        if terrain.is_valid() {
            // section[0]=terrain, section[1]=interactive objects.
            let sprite = extract_sprite(&data[0], terrain.width, terrain.height, terrain.image_loc, terrain.mask_loc);
            println!("{}: {:?}", i, terrain);
        }
    }

    Ok(())
}
