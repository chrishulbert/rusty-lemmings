// This loads the games from disk into memory.

use std::env;
use std::fs;
use std::io::Result;
use std::path::Path;

use crate::lemmings::models::*;
use crate::lemmings::parsers::*;

// Load a ground file and its associated vga graphics.
fn load_ground_and_sprites(dir: &str, index: i32) -> Result<GroundCombined> {
    let vga_file: Vec<u8> = fs::read(format!("{}/vgagr{}.dat", dir, index))?;
    let vga_sections = decompressor::decompress(&vga_file)?;

    let ground_file: Vec<u8> = fs::read(format!("{}/ground{}o.dat", dir, index))?;
    let ground = ground::parse(&ground_file)?;
    let palette = ground.palettes.as_abgr();

    let mut terrain_sprites: ImageMap = ImageMap::new();
    for (i, terrain) in ground.terrain_info.iter().enumerate() {
        if terrain.is_valid() {
            let sprite = sprites::extract_image(&vga_sections[0], terrain.width, terrain.height, terrain.image_loc, terrain.mask_loc, &palette);
            terrain_sprites.insert(i as i32, sprite);
        }
    }

    let mut object_sprites: AnimationMap = AnimationMap::new();
    for (i, object) in ground.object_info.iter().enumerate() {
        if object.is_valid() {
            // TODO do we need +1 for the # of frames?
            let sprite = sprites::extract_animation(&vga_sections[1], object.width, object.height, object.animation_frames_base_loc as usize, object.animation_frames_base_loc as usize + object.mask_offset_from_image as usize, &palette, object.animation_frame_data_size as usize, object.end_animation_frame_index as usize);
            object_sprites.insert(i as i32, sprite);
        }
    }

    Ok(GroundCombined {
        ground: ground,
        terrain_sprites: terrain_sprites,
        object_sprites: object_sprites,
    })
}

fn load_all_grounds(dir: &str) -> Result<GroundMap> {
    let mut all: GroundMap = GroundMap::new();
    for entry in fs::read_dir(dir)? {
        if let Ok(entry) = entry {
            let raw_name = entry.file_name().into_string().unwrap();
            let file_name = raw_name.to_lowercase();
            if file_name.starts_with("vgagr") && file_name.ends_with(".dat") {
                let file_number: i32 = file_name[5..6].parse().unwrap();
                let ground = load_ground_and_sprites(dir, file_number)?;
                all.insert(file_number, ground);
            }
        }
    }
    Ok(all)
}

fn load_all_specials(dir: &str) -> Result<SpecialMap> {
    let mut all: SpecialMap = SpecialMap::new();
    for entry in fs::read_dir(dir)? {
        if let Ok(entry) = entry {
            let raw_name = entry.file_name().into_string().unwrap();
            let file_name = raw_name.to_lowercase();
            if file_name.starts_with("vgaspec") && file_name.ends_with(".dat") {
                let file_number: i32 = file_name[7..8].parse().unwrap();
                let filename = format!("{}/{}", dir, raw_name);
                let raw: Vec<u8> = fs::read(filename)?;
                let sections = decompressor::decompress(&raw)?;
                let spec = special::parse(&sections[0])?;
                all.insert(file_number, spec);
            }
        }
    }
    Ok(all)
}

// Load all the levels from all the sections in all the files into memory.
// Might as well load into ram, only takes <30ms on my laptop in release mode.
fn load_all_levels(dir: &str) -> Result<LevelMap> {
    let mut all: LevelMap = LevelMap::new();
    for entry in fs::read_dir(dir)? {
        if let Ok(entry) = entry {
            let raw_name = entry.file_name().into_string().unwrap();
            let file_name = raw_name.to_lowercase();
            if (file_name.starts_with("level") || file_name.starts_with("dlvel")) && file_name.ends_with(".dat") {
                let file_number: i32 = file_name[5..8].parse().unwrap();
                let filename = format!("{}/{}", dir, raw_name);
                let raw: Vec<u8> = fs::read(filename)?;
                let sections = decompressor::decompress(&raw)?;
                for (section_index, section) in sections.iter().enumerate() {
                    let level = level::parse(section)?;
                    let key = file_number*100 + (section_index as i32);
                    all.insert(key, level);
                }
            }
        }
    }
    Ok(all)
}

fn load_main_dat(dir: &str) -> Result<MainDat> {
    let file: Vec<u8> = fs::read(format!("{}/main.dat", dir))?;
    let sections = decompressor::decompress(&file)?;
    maindat::parse(&sections)
}

fn load_game(dir: &str, sub_dir: &str, name: &str) -> Result<Option<Game>> {
    let sub_path = format!("{}/{}", dir, sub_dir);
    if !Path::new(&sub_path).exists() {
        return Ok(None);
    }
    Ok(Some(Game {
        name: name.to_string(),
        id: sub_dir.to_string(),
        levels: load_all_levels(&sub_path)?,
        specials: load_all_specials(&sub_path)?,
        grounds: load_all_grounds(&sub_path)?,
        main: load_main_dat(&sub_path)?,
    }))
}

pub fn load() -> Result<Games> {
    let home = env::var("HOME").unwrap_or("~".to_string());
    let data_root = format!("{}/Lemmings", home);
    return Ok(Games {
        lemmings: load_game(&data_root, "lemmings", "Lemmings")?,
        oh_no_more: load_game(&data_root, "ohnomore", "Oh no! More Lemmings")?,
        christmas_91: load_game(&data_root, "christmas1991", "Xmas Lemmings '91")?,
        christmas_92: load_game(&data_root, "christmas1992", "Xmas Lemmings '92")?,
        holiday_93: load_game(&data_root, "holiday1993", "Holiday Lemmings '93")?,
        holiday_94: load_game(&data_root, "holiday1994", "Holiday Lemmings '94")?,
    })
}
