use std::fs;
use std::io;
use std::mem;
use std::cmp;
use std::slice;
use std::collections::HashMap;
use std::env;

extern crate image;

mod lemmings;
use lemmings::parsers::*;
use lemmings::models::*;

fn data_dir() -> String {
    return format!("{}/Lemmings/lemmings", env::home_dir().unwrap().to_str().unwrap());
}

fn u32_to_u8_slice(original: &[u32]) -> &[u8] {
    let count = original.len() * mem::size_of::<u32>();
    let ptr = original.as_ptr() as *const u8;
    return unsafe { slice::from_raw_parts(ptr, count) };
}

struct GroundCombined {
    ground: Ground,
    terrain_sprites: HashMap<usize, Vec<u32>>,
    object_sprites: HashMap<usize, Vec<u32>>, // TODO animations.
}

fn load_ground_and_sprites(index: u8) -> io::Result<GroundCombined> {
    let vga_file: Vec<u8> = fs::read(format!("{}/vgagr{}.dat", data_dir(), index))?;
    let vga_sections = decompressor::decompress(&vga_file)?;

    let ground_file: Vec<u8> = fs::read(format!("{}/ground{}o.dat", data_dir(), index))?;
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

fn load_all_grounds() -> io::Result<Vec<GroundCombined>> {
    let mut all: Vec<GroundCombined> = Vec::new();
    for i in 0..5 { // TODO load by file search
        let ground = load_ground_and_sprites(i)?;
        all.push(ground);
    }
    Ok(all)
}

type SpecialMap = HashMap<i32, Image>;

fn load_all_specials() -> io::Result<SpecialMap> {
    let mut all: SpecialMap = SpecialMap::new();
    for entry in fs::read_dir(data_dir())? {
        if let Ok(entry) = entry {
            let raw_name = entry.file_name().into_string().unwrap();
            let file_name = raw_name.to_lowercase();
            if file_name.starts_with("vgaspec") && file_name.ends_with(".dat") {
                let file_number: i32 = file_name[7..8].parse().unwrap();
                let filename = format!("{}/{}", data_dir(), raw_name);
                let raw: Vec<u8> = fs::read(filename)?;
                let sections = decompressor::decompress(&raw)?;
                let spec = special::parse(&sections[0])?;
                all.insert(file_number, spec);
            }
        }
    }
    Ok(all)
}

// Key is file# * 100 + section. Eg 203 = LEVEL002.DAT section 3.
type LevelMap = HashMap<usize, Level>;

// Load all the levels from all the sections in all the files into memory.
// Might as well load into ram, only takes <30ms on my laptop in release mode.
fn load_all_levels() -> io::Result<LevelMap> {
    let mut all: LevelMap = LevelMap::new();
    for entry in fs::read_dir(data_dir())? {
        if let Ok(entry) = entry {
            let raw_name = entry.file_name().into_string().unwrap();
            let file_name = raw_name.to_lowercase();
            if file_name.starts_with("level") && file_name.ends_with(".dat") {
                let file_number: usize = file_name[5..8].parse().unwrap();
                let filename = format!("{}/{}", data_dir(), raw_name);
                let raw: Vec<u8> = fs::read(filename)?;
                let sections = decompressor::decompress(&raw)?;
                for (section_index, section) in sections.iter().enumerate() {
                    let level = level::parse(section)?;
                    let key = file_number*100 + section_index;
                    all.insert(key, level);
                }
            }
        }
    }
    Ok(all)
}

#[derive(Debug, Copy, Clone)]
struct LevelSize {
    min_x: i32,
    max_x: i32,
}

impl LevelSize {
    fn width(&self) -> i32 {
        self.max_x - self.min_x
    }
}

const SPECIAL_LEFT_X: i32 = 320;

fn size_of_level(level: &Level, grounds: &[GroundCombined]) -> LevelSize {
    if level.globals.extended_graphic_set != 0 {
        return LevelSize {
            min_x: SPECIAL_LEFT_X,
            max_x: SPECIAL_LEFT_X + special::WIDTH as i32,            
        }
    }

    let mut size = LevelSize {
        min_x: std::i32::MAX,
        max_x: std::i32::MIN,
    };
    let ground = &grounds[level.globals.normal_graphic_set as usize];
    for terrain in level.terrain.iter() {
        let width = ground.ground.terrain_info[terrain.terrain_id as usize].width as i32;
        size.min_x = cmp::min(size.min_x, terrain.x);
        size.max_x = cmp::max(size.max_x, terrain.x + width);
    }
    size
}

struct RenderedLevel {
    bitmap: Vec<u32>,
    size: LevelSize,
}

const LEVEL_BACKGROUND: u32 = 0xff000000;
const LEVEL_HEIGHT: i32 = 160;

fn draw(sprite: &Vec<u32>,
        x: i32, y: i32,
        sprite_width: i32, sprite_height: i32,
        canvas: &mut Vec<u32>, canvas_width: i32, canvas_height: i32,
        do_not_overwrite_existing_terrain: bool,
        is_upside_down: bool,
        remove_terrain: bool,
        must_have_terrain_underneath_to_be_visible: bool) {
    let mut canvas_offset = y * canvas_width + x;
    let canvas_stride = canvas_width - sprite_width;
    let mut sprite_offset: i32 = if is_upside_down { (sprite_height - 1) * sprite_width } else { 0 };
    let sprite_stride: i32 = if is_upside_down { -2 * sprite_width } else { 0 };
    for pixel_y in 0..sprite_height {
        if pixel_y+y < 0 || pixel_y+y >= canvas_height { // Out of bounds, skip a row.
            canvas_offset += sprite_width + canvas_stride;
            sprite_offset += sprite_width + sprite_stride;
            continue
        }

        for pixel_x in 0..sprite_width {
            if pixel_x+x < 0 || pixel_x+x >= canvas_width { // Out of canvas bounds, skip this pixel.
                sprite_offset += 1;
                canvas_offset += 1;
                continue
            }

            if remove_terrain {
                if sprite[sprite_offset as usize] != 0 {
                    canvas[canvas_offset as usize] = LEVEL_BACKGROUND;
                }
                sprite_offset += 1;
                canvas_offset += 1;
                continue;
            }
            if do_not_overwrite_existing_terrain {
                if canvas[canvas_offset as usize] != LEVEL_BACKGROUND { // Skip the 'paint' if there's existing terrain.
                    sprite_offset += 1;
                    canvas_offset += 1;
                    continue;
                }
            }
            if must_have_terrain_underneath_to_be_visible {
                if canvas[canvas_offset as usize] == LEVEL_BACKGROUND { // Skip the 'paint' if there's no existing terrain.
                    sprite_offset += 1;
                    canvas_offset += 1;
                    continue;
                }
            }
            let pixel = sprite[sprite_offset as usize];
            if pixel != 0 {
                canvas[canvas_offset as usize] = pixel;
            }
            sprite_offset += 1;
            canvas_offset += 1;
        }
        canvas_offset += canvas_stride;
        sprite_offset += sprite_stride;
    }
}

fn render_level(level: &Level, grounds: &[GroundCombined], specials: &SpecialMap) -> io::Result<RenderedLevel> {
    let size = size_of_level(level, grounds);
    let width = size.width();
    let height = LEVEL_HEIGHT;
    let pixels = width * height;
    let mut rendered_level = RenderedLevel {
        bitmap: vec![LEVEL_BACKGROUND; pixels as usize],
        size: size,
    };
    let ground = &grounds[level.globals.normal_graphic_set as usize];
    if level.globals.extended_graphic_set == 0 {
        for terrain in level.terrain.iter() {
            let terrain_info = &ground.ground.terrain_info[terrain.terrain_id as usize];
            let sprite = &ground.terrain_sprites[&terrain.terrain_id];
            draw(&sprite,
                (terrain.x - size.min_x) as i32, terrain.y,
                terrain_info.width as i32, terrain_info.height as i32,
                &mut rendered_level.bitmap,
                width as i32, height as i32,
                terrain.do_not_overwrite_existing_terrain,
                terrain.is_upside_down,
                terrain.remove_terrain,
                false);
        }
    } else {
        let special = &specials[&(level.globals.extended_graphic_set as i32 - 1)];
        rendered_level.bitmap.copy_from_slice(&special.bitmap);
    }
    for object in level.objects.iter() {
        let object_info = &ground.ground.object_info[object.obj_id as usize];
        let sprite = &ground.object_sprites[&object.obj_id];
        draw(&sprite,
            (object.x - size.min_x) as i32, object.y as i32,
            object_info.width as i32, object_info.height as i32,
            &mut rendered_level.bitmap,
            width as i32, height as i32,
            object.modifier.is_do_not_overwrite_existing_terrain(),
            object.is_upside_down,
            false,
            object.modifier.is_must_have_terrain_underneath_to_be_visible());
    }
    Ok(rendered_level)
}

fn main() -> io::Result<()> {
    use std::time::Instant;
    let now = Instant::now();
    println!("Loading levels");
    let levels = load_all_levels()?;
    println!("Loading grounds");
    let grounds = load_all_grounds()?;
    println!("Loading specials");
    let specials = load_all_specials()?;
    let elapsed = now.elapsed();
    println!("Took: {:?}", elapsed); // 27ms optimised.

    for (key, level) in &levels {
        // let key = 1;
        // let level = &levels[&key];
        // println!("Level: {:?}", level);
        let rendered = render_level(level, &grounds, &specials)?;
        let buf = u32_to_u8_slice(&rendered.bitmap);
        let filename = format!("output/levels/{} {} ({} - {}).png", key, level.name, level.globals.normal_graphic_set, level.globals.extended_graphic_set);
        image::save_buffer(filename, &buf, rendered.size.width() as u32, LEVEL_HEIGHT as u32, image::RGBA(8)).unwrap();
    }

    // for i in 0..10 {
    //     extract_level(i, &grounds)?;
    // }

    // for i in 0..5 {
    //     extract(i)?;
    // }
    // for i in 0..4 {
    //     extract_special(i)?;
    // }

    // Main.dat
    // let main_raw: Vec<u8> = fs::read("{}/main.dat", data_dir())?;
    // let main_sections = decompressor::decompress(&main_raw)?;
    // let main = maindat::parse(&main_sections)?;
    // let image = main.main_menu.logo;
    // let buf = u32_to_u8_slice(&image.bitmap);
    // image::save_buffer("output/background.png", &buf, image.width as u32, image.height as u32, image::RGBA(8)).unwrap();

    Ok(())
}
