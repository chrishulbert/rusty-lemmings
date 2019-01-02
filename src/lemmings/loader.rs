// This loads the games from disk into memory.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Result;
use std::path::Path;

use lemmings::models::*;
use lemmings::parsers::*;

struct GroundCombined {
    ground: Ground,
    terrain_sprites: HashMap<usize, Vec<u32>>,
    object_sprites: HashMap<usize, Vec<u32>>, // TODO animations.
}

type GroundMap = HashMap<i32, GroundCombined>;
type LevelMap = HashMap<i32, Level>; // Key is file# * 100 + section. Eg 203 = LEVEL002.DAT section 3.
type SpecialMap = HashMap<i32, Image>;

pub struct Game {
    pub levels: LevelMap,
    pub specials: SpecialMap,
    pub grounds: GroundMap,
}

pub struct Games {
    pub lemmings: Option<Game>,
    pub oh_no_more: Option<Game>,
    pub christmas_91: Option<Game>,
    pub christmas_92: Option<Game>,
    pub holiday_93: Option<Game>,
    pub holiday_94: Option<Game>,
}

// Load a ground file and its associated vga graphics.
fn load_ground_and_sprites(dir: &str, index: i32) -> Result<GroundCombined> {
    let vga_file: Vec<u8> = fs::read(format!("{}/vgagr{}.dat", dir, index))?;
    let vga_sections = decompressor::decompress(&vga_file)?;

    let ground_file: Vec<u8> = fs::read(format!("{}/ground{}o.dat", dir, index))?;
    let ground = ground::parse(&ground_file)?;
    let palette = ground.palettes.as_abgr();

    let mut terrain_sprites: HashMap<usize, Vec<u32>> = HashMap::new();
    for (i, terrain) in ground.terrain_info.iter().enumerate() {
        if terrain.is_valid() {
            let sprite = sprites::extract(&vga_sections[0], terrain.width, terrain.height, terrain.image_loc, terrain.mask_loc, &palette);
            terrain_sprites.insert(i, sprite);
        }
    }

    let mut object_sprites: HashMap<usize, Vec<u32>> = HashMap::new();
    for (i, object) in ground.object_info.iter().enumerate() {
        if object.is_valid() {
            let sprite = sprites::extract(&vga_sections[1], object.width, object.height, object.animation_frames_base_loc, object.animation_frames_base_loc + object.mask_offset_from_image, &palette);
            object_sprites.insert(i, sprite);
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
            if file_name.starts_with("level") && file_name.ends_with(".dat") {
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

fn load_game(dir: &str, sub_dir: &str) -> Result<Option<Game>> {
    let sub_path = format!("{}/{}", dir, sub_dir);
    if !Path::new(&sub_path).exists() {
        return Ok(None);
    }
    Ok(Some(Game {
        levels: load_all_levels(&sub_path)?,
        specials: load_all_specials(&sub_path)?,
        grounds: load_all_grounds(&sub_path)?,
    }))
}

pub fn load() -> Result<Games> {
    let data_root = format!("{}/Lemmings", env::home_dir().unwrap().to_str().unwrap());
    return Ok(Games {
        lemmings: load_game(&data_root, "lemmings")?,
        oh_no_more: load_game(&data_root, "ohnomore")?,
        christmas_91: load_game(&data_root, "christmas1991")?,
        christmas_92: load_game(&data_root, "christmas1992")?,
        holiday_93: load_game(&data_root, "holiday1993")?,
        holiday_94: load_game(&data_root, "holiday1994")?,
    })
}