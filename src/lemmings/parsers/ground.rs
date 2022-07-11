// This is for parsing lemmings GROUND files:
// https://www.camanis.net/lemmings/files/docs/lemmings_vgagrx_dat_groundxo_dat_file_format.txt

use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::slice;
use crate::lemmings::models::*;

impl Default for Ground {
    fn default() -> Ground {
        Ground {
            object_info: Default::default(),
            terrain_info: [Default::default(); 64], // Default only auto-derives up to 32 element arrays.
            palettes: Default::default(),
        }
    }
}

// Unlike the .LVL file format, WORDs in groundXo.dat are stored little-endian (camanis.net).
fn read_u16(data: &mut slice::Iter<u8>) -> u16 {
    let little = *data.next().unwrap();
    let big = *data.next().unwrap();
    return ((big as u16) << 8) + (little as u16);
}

// Upgrades a 6-bit colour to 8, while still allowing 100% black and white.
fn colour_upgrade(six: u8) -> u8 {
    if six == 0 { 0 } else { (six << 2) + 3 }
}

// Read 3 RGB bytes, converting to 0-255 format.
// Source file: (0x3F, 0x00, 0x00) gives you the brightest red you can get (camanis.net)
fn read_rgb(data: &mut slice::Iter<u8>) -> u32 {
    let r6 = *data.next().unwrap();
    let g6 = *data.next().unwrap();
    let b6 = *data.next().unwrap();
    let r8: u8 = colour_upgrade(r6);
    let g8: u8 = colour_upgrade(g6);
    let b8: u8 = colour_upgrade(b6);
    return ((r8 as u32) << 16) + ((g8 as u32) << 8) + (b8 as u32);
}

/// Parses a ground file.
pub fn parse(data: &[u8]) -> io::Result<Ground> {
    if data.len() != 1056 {
        return Err(Error::new(ErrorKind::InvalidData, "Ground data wrong length"))
    }
    let mut ground: Ground = Default::default();
    let mut data_iter = data.into_iter();
    for i in 0..16 {
        ground.object_info[i].animation_flags = read_u16(&mut data_iter);
        ground.object_info[i].start_animation_frame_index = *data_iter.next().unwrap();
        ground.object_info[i].end_animation_frame_index = *data_iter.next().unwrap();
        ground.object_info[i].width = *data_iter.next().unwrap() as usize;
        ground.object_info[i].height = *data_iter.next().unwrap() as usize;
        ground.object_info[i].animation_frame_data_size = read_u16(&mut data_iter);
        ground.object_info[i].mask_offset_from_image = read_u16(&mut data_iter);
        let _unknown1 = read_u16(&mut data_iter);
        let _unknown2 = read_u16(&mut data_iter);
        ground.object_info[i].trigger_left = read_u16(&mut data_iter);
        ground.object_info[i].trigger_top = read_u16(&mut data_iter);
        ground.object_info[i].trigger_width = *data_iter.next().unwrap();
        ground.object_info[i].trigger_height = *data_iter.next().unwrap();
        ground.object_info[i].trigger_effect_id = *data_iter.next().unwrap();
        ground.object_info[i].animation_frames_base_loc = read_u16(&mut data_iter);
        ground.object_info[i].preview_image_index = read_u16(&mut data_iter);
        let _unknown3 = read_u16(&mut data_iter);
        ground.object_info[i].trap_sound_effect_id = *data_iter.next().unwrap();
    }
    for i in 0..64 {
        ground.terrain_info[i].width = *data_iter.next().unwrap() as usize;
        ground.terrain_info[i].height = *data_iter.next().unwrap() as usize;
        ground.terrain_info[i].image_loc = read_u16(&mut data_iter);
        ground.terrain_info[i].mask_loc = read_u16(&mut data_iter);
        let _unknown = read_u16(&mut data_iter);
    }
    for i in 0..8 {
        ground.palettes.ega_custom[i] = *data_iter.next().unwrap();
    }
    for i in 0..8 {
        ground.palettes.ega_standard[i] = *data_iter.next().unwrap();
    }
    for i in 0..8 {
        ground.palettes.ega_preview[i] = *data_iter.next().unwrap();
    }
    for i in 0..8 {
        ground.palettes.vga_custom[i] = read_rgb(&mut data_iter);
    }
    for i in 0..8 {
        ground.palettes.vga_standard[i] = read_rgb(&mut data_iter);
    }
    for i in 0..8 {
        ground.palettes.vga_preview[i] = read_rgb(&mut data_iter);
    }
    Ok(ground)
}
