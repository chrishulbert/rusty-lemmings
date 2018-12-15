// This is for parsing lemmings GROUND files:
// https://www.camanis.net/lemmings/files/docs/lemmings_vgagrx_dat_groundxo_dat_file_format.txt

use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::slice;

#[derive(Default)]
pub struct ObjectInfo {
    pub animation_flags: u16,
    pub start_animation_frame_index: u8,
    pub end_animation_frame_index: u8,
    pub width: u8,
    pub height: u8,
    pub animation_frame_data_size: u16,
    pub mask_offset_from_image: u16,
    unknown1: u16,
    unknown2: u16,
    pub trigger_left: u16,
    pub trigger_top: u16,
    pub trigger_width: u8,
    pub trigger_height: u8,
    pub trigger_effect_id: u8,
    pub animation_frames_base_loc: u16,
    pub preview_image_index: u16,
    unknown3: u16,
    pub trap_sound_effect_id: u8,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct TerrainInfo {
    pub width: u8,
    pub height: u8,
    pub image_loc: u16,
    pub mask_loc: u16,
    unknown1: u16,
}

impl TerrainInfo {
    pub fn is_valid(&self) -> bool {
        return self.width>0 || self.height>0 || self.image_loc>0 || self.mask_loc>0;
    }
}

#[derive(Default)]
pub struct Palettes {
    pub ega_custom: [u8; 8],
    pub ega_standard: [u8; 8],
    pub ega_preview: [u8; 8],
    pub vga_custom: [u32; 8], // RGB Palette entries 8...15. Only 6 bits so 0x3f = 100%
    pub vga_standard: [u32; 8], // Doesn't seem to be used by the game.
    pub vga_preview: [u32; 8],
}

pub struct Ground {
    pub object_info: [ObjectInfo; 16],
    pub terrain_info: [TerrainInfo; 64],
    pub palettes: Palettes,
}

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

// Read 3 RGB bytes.
// (0x3F, 0x00, 0x00) gives you the brightest red you can get (camanis.net)
fn read_rgb(data: &mut slice::Iter<u8>) -> u32 {
    let r = *data.next().unwrap();
    let g = *data.next().unwrap();
    let b = *data.next().unwrap();
    return ((r as u32) << 16) + ((g as u32) << 8) + (b as u32);
}

/// Decompresses all the sections from a compressed dat file.
/// Returns a vec of sections. Each section is a vec of its data.
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
        ground.object_info[i].width = *data_iter.next().unwrap();
        ground.object_info[i].height = *data_iter.next().unwrap();
        ground.object_info[i].animation_frame_data_size = read_u16(&mut data_iter);
        ground.object_info[i].mask_offset_from_image = read_u16(&mut data_iter);
        ground.object_info[i].unknown1 = read_u16(&mut data_iter);
        ground.object_info[i].unknown2 = read_u16(&mut data_iter);
        ground.object_info[i].trigger_left = read_u16(&mut data_iter);
        ground.object_info[i].trigger_top = read_u16(&mut data_iter);
        ground.object_info[i].trigger_width = *data_iter.next().unwrap();
        ground.object_info[i].trigger_height = *data_iter.next().unwrap();
        ground.object_info[i].trigger_effect_id = *data_iter.next().unwrap();
        ground.object_info[i].animation_frames_base_loc = read_u16(&mut data_iter);
        ground.object_info[i].preview_image_index = read_u16(&mut data_iter);
        ground.object_info[i].unknown3 = read_u16(&mut data_iter);
        ground.object_info[i].trap_sound_effect_id = *data_iter.next().unwrap();
    }
    for i in 0..64 {
        ground.terrain_info[i].width = *data_iter.next().unwrap();
        ground.terrain_info[i].height = *data_iter.next().unwrap();
        ground.terrain_info[i].image_loc = read_u16(&mut data_iter);
        ground.terrain_info[i].mask_loc = read_u16(&mut data_iter);
        ground.terrain_info[i].unknown1 = read_u16(&mut data_iter);
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
