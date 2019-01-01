// This loads the special levels eg VGASPEC0.DAT.
// https://www.camanis.net/lemmings/files/docs/lemmings_vgaspecx_dat_file_format.txt

use std::io::{Error, ErrorKind, Result};
use std::slice::Iter;
use lemmings::models::*;
use super::helpers::BitsIterMS;

// Creates a bit iterator from [u8].
macro_rules! iterate_bits { ($data:expr) => { $data.iter().flat_map(BitsIterMS::new); } }

// Reads a byte, failing gracefully if none are left.
fn read_u8(data: &mut Iter<u8>) -> Result<u8> {
    match data.next() {
        Some(t) => Ok(*t),
        None => Err(Error::new(ErrorKind::UnexpectedEof, "No data remaining")),
    }
}

// Upgrades a 6-bit colour to 8, while still allowing 100% black and white.
#[inline]
fn colour_upgrade(six: u8) -> u8 {
    if six == 0 { 0 } else { (six << 2) + 3 }
}

// Read 3 RGB bytes, outputting ABGR.
// (0x3F, 0x00, 0x00) gives you the brightest red you can get (camanis.net)
fn read_rgb(data: &mut Iter<u8>) -> Result<u32> {
    let r6 = read_u8(data)?;
    let g6 = read_u8(data)?;
    let b6 = read_u8(data)?;
    let r8: u8 = colour_upgrade(r6);
    let g8: u8 = colour_upgrade(g6);
    let b8: u8 = colour_upgrade(b6);
    Ok(0xff000000 + ((b8 as u32) << 16) + ((g8 as u32) << 8) + (r8 as u32))
}

pub const WIDTH: usize = 960;
const HEIGHT: usize = 160;
const PIXELS: usize = WIDTH * HEIGHT;
const SECTION_HEIGHT: usize = 40;
const SECTION_PIXELS: usize = WIDTH * SECTION_HEIGHT;
const SECTION_CAPACITY: usize = SECTION_PIXELS * 3 / 8; // Decompressed quarter-section size in bytes.

// Pass this data that has already been DAT-decompressed.
pub fn parse(data: &[u8]) -> Result<Image> {
    let mut iter = data.into_iter();

    // Palette.
    let mut palette: [u32; 8] = [0; 8];
    let _unused_first = read_rgb(&mut iter)?; // First palette entry is replaced with black.
    palette[0] = 0xff000000;
    for i in 1..8 {
        palette[i] = read_rgb(&mut iter)?;
    }
    for _ in 0..16 { let _ega = read_u8(&mut iter); } // Discard EGA.

    // RLE-decompress it.
    let mut bitmap: Vec<u32> = Vec::with_capacity(PIXELS);
    let mut decompressed: Vec<u8> = Vec::with_capacity(SECTION_CAPACITY);
    while bitmap.len() < PIXELS {
        let byte = read_u8(&mut iter)?;
        if byte <= 0x7f { // Raw chunk.
            let count = byte + 1;
            for _ in 0..count {
                let b = read_u8(&mut iter)?;
                decompressed.push(b);
            }
        } else if byte >= 0x81 { // Run chunk.
            let count = (255 - byte) + 2;
            let b = read_u8(&mut iter)?;
            for _ in 0..count {
                decompressed.push(b);
            }
        } else { // End of section. Each section should be of size 14400 bytes.
            { // Apply this section to the pixels. Braces to scope the iterators.
                let mut image_iter_0 = iterate_bits!(decompressed);
                let mut image_iter_1 = iterate_bits!(decompressed).skip(SECTION_PIXELS);
                let mut image_iter_2 = iterate_bits!(decompressed).skip(SECTION_PIXELS * 2);
                for _ in 0..SECTION_PIXELS {
                    let colour_index =
                        image_iter_0.next().unwrap() +
                        (image_iter_1.next().unwrap() << 1) +
                        (image_iter_2.next().unwrap() << 2);
                    let colour = palette[colour_index as usize];
                    bitmap.push(colour);
                }
            }
            decompressed.clear(); // This happily keeps the capacity.
        }
    }

    Ok(Image {
        width: WIDTH,
        height: HEIGHT,
        bitmap: bitmap,
    })
}
