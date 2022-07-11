// This extracts sprites from data.

use super::helpers::BitsIterMS;

use crate::lemmings::models::*;

// Creates a bit iterator from [u8].
macro_rules! iterate_bits { ($data:expr) => { $data.iter().flat_map(BitsIterMS::new) } }

// Extract a single sprite.
// Sprites are stored as 4 planes, eg all the 1 bits, then all the 2 bits, then so on. It seems they did this so
// they could reuse one of the bits as the mask plane. But it means parsing is weird.
// Palette entries are 0xRRGGBBAA
fn extract_frame(data: &[u8], width: usize, height: usize, image_loc: usize, mask_loc: usize, palette: &[u32; 16]) -> Vec<u32> {
    let image_data: &[u8] = &data[image_loc..];
    let mask_data: &[u8] = &data[mask_loc..];
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
        let masked_colour: u32 = if mask_iter.next().unwrap() == 0 { 0 } else { colour };
        sprite.push(masked_colour);
    }
    sprite
}

pub fn extract_image(data: &[u8], width: usize, height: usize, image_loc: u16, mask_loc: u16, palette: &[u32; 16]) -> Image {
    Image {
        bitmap: extract_frame(data, width, height, image_loc as usize, mask_loc as usize, palette),
        width: width,
        height: height,
    }
}

pub fn extract_animation(data: &[u8], width: usize, height: usize, image_loc: usize, mask_loc: usize, palette: &[u32; 16], stride: usize, frame_count: usize) -> Animation {
    let mut frames: Vec<Vec<u32>> = Vec::new();
    for i in 0..frame_count {
        let offset = stride * i;
        let frame = extract_frame(data, width, height, offset + image_loc, offset + mask_loc, palette);
        frames.push(frame);
    }
    Animation {
        frames: frames,
        width: width,
        height: height,
    }
}
