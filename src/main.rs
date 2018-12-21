use std::fs;
use std::io;
use std::mem;
use std::cmp;
use std::slice;
use std::collections::HashMap;

extern crate image;

mod lemmings;
use lemmings::data::{ maindat, special, decompressor, ground, sprites, level };

fn u32_to_u8_slice(original: &[u32]) -> &[u8] {
    let count = original.len() * mem::size_of::<u32>();
    let ptr = original.as_ptr() as *const u8;
    return unsafe { slice::from_raw_parts(ptr, count) };
}

struct GroundCombined {
    ground: ground::Ground,
    terrain_sprites: HashMap<usize, Vec<u32>>,
    object_sprites: HashMap<usize, Vec<u32>>, // TODO animations.
}

fn load_ground_and_sprites(index: u8) -> io::Result<GroundCombined> {
    let vga_file: Vec<u8> = fs::read(format!("data/VGAGR{}.DAT", index))?;
    let vga_sections = decompressor::decompress(&vga_file)?;

    let ground_file: Vec<u8> = fs::read(format!("data/GROUND{}O.DAT", index))?;
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

// Key is file# * 100 + section. Eg 203 = LEVEL002.DAT section 3.
type LevelMap = HashMap<usize, level::Level>;

// Load all the levels from all the sections in all the files into memory.
// Might as well load into ram, only takes <30ms on my laptop in release mode.
fn load_all_levels() -> io::Result<LevelMap> {
    let mut all: LevelMap = LevelMap::new();
    for entry in fs::read_dir("data")? {
        if let Ok(entry) = entry {
            let raw_name = entry.file_name().into_string().unwrap();
            let file_name = raw_name.to_lowercase();
            if file_name.starts_with("level") && file_name.ends_with(".dat") {
                let file_number: usize = file_name[5..8].parse().unwrap();
                let filename = format!("data/{}", raw_name);
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

fn extract_level(index: isize, grounds: &[GroundCombined]) -> io::Result<()> {
    println!("Extracting: {}", index);
    fs::create_dir_all(format!("output/levels/{}", index))?;

    let filename = format!("data/LEVEL00{}.DAT", index);
    let raw: Vec<u8> = fs::read(filename)?;

    let sections = decompressor::decompress(&raw)?;
    println!("Sections: {:?}", sections.len());

    for (i, section) in sections.iter().enumerate() {
        fs::write(format!("output/levels/{}/{}", index, i), section)?;

        let level = level::parse(section)?;
        println!("{}: {} (num_of_lemmings = {})", i, level.name, level.globals.num_of_lemmings);

        let size = size_of_level(&level, grounds);
        println!("size: {:?}", size);
    }

    Ok(())
}

fn extract_special(index: isize) -> io::Result<()> {
    let spec_raw: Vec<u8> = fs::read(format!("data/VGASPEC{}.DAT", index))?;
    let spec_sections = decompressor::decompress(&spec_raw)?;
    // if spec_sections.len() != 0 {
    //     return Err(Error::new(ErrorKind::InvalidData, "Wrong section count"));
    // }
    let spec = special::parse(&spec_sections[0])?;
    let buf = u32_to_u8_slice(&spec.bitmap);
    image::save_buffer(format!("output/spec{}.png", index), &buf, spec.width as u32, spec.height as u32, image::RGBA(8)).unwrap();

    Ok(())
}

#[derive(Debug, Copy, Clone)]
struct Rect {
    min_x: isize,
    min_y: isize,
    max_x: isize,
    max_y: isize,
}

impl Rect {
    fn width(&self) -> usize {
        (self.max_x - self.min_x) as usize
    }
    fn height(&self) -> usize {
        (self.max_y - self.min_y) as usize
    }
}

fn size_of_level(level: &level::Level, grounds: &[GroundCombined]) -> Rect {
    let mut rect = Rect {
        min_x: std::isize::MAX,
        min_y: std::isize::MAX,
        max_x: std::isize::MIN,
        max_y: std::isize::MIN,
    };
    let ground = &grounds[level.globals.normal_graphic_set as usize];
    for object in level.objects.iter() {
        let width = ground.ground.object_info[object.obj_id as usize].width as isize;
        let height = ground.ground.object_info[object.obj_id as usize].height as isize;
        rect.min_x = cmp::min(rect.min_x, object.x);
        rect.min_y = cmp::min(rect.min_y, object.y);
        rect.max_x = cmp::max(rect.max_x, object.x + width);
        rect.max_y = cmp::max(rect.max_y, object.y + height);
    }
    for terrain in level.terrain.iter() {
        let width = ground.ground.terrain_info[terrain.terrain_id as usize].width as isize;
        let height = ground.ground.terrain_info[terrain.terrain_id as usize].height as isize;
        rect.min_x = cmp::min(rect.min_x, terrain.x);
        rect.min_y = cmp::min(rect.min_y, terrain.y);
        rect.max_x = cmp::max(rect.max_x, terrain.x + width);
        rect.max_y = cmp::max(rect.max_y, terrain.y + height);
    }
    rect
}

struct RenderedLevel {
    bitmap: Vec<u32>,
    rect: Rect,
}

const LEVEL_BACKGROUND: u32 = 0xff000000;

fn draw(sprite: &Vec<u32>, x: i32, y: i32, sprite_width: i32, sprite_height: i32, canvas: &mut Vec<u32>, canvas_width: i32, canvas_height: i32,
        do_not_overwrite_existing_terrain: bool,
        is_upside_down: bool,
        remove_terrain: bool) {
    let mut canvas_offset = y as i32 * canvas_width + x as i32;
    let canvas_stride = canvas_width - sprite_width;
    let mut sprite_offset: i32 = if is_upside_down { (sprite_height - 1) * sprite_width } else { 0 };
    let sprite_stride: i32 = if is_upside_down { -2 * sprite_width } else { 0 };
    if remove_terrain {
        println!("Remove!??");
    }
    for _ in 0..sprite_height {
        for _ in 0..sprite_width {
            if do_not_overwrite_existing_terrain {
                if canvas[canvas_offset as usize] != LEVEL_BACKGROUND {
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

fn render_level(level: &level::Level, grounds: &[GroundCombined]) -> io::Result<RenderedLevel> {
    let rect = size_of_level(level, grounds);
    let width = rect.width();
    let height = rect.height();
    let pixels = (width * height) as usize;
    let mut rendered_level = RenderedLevel {
        bitmap: vec![LEVEL_BACKGROUND; pixels],
        rect: rect,
    };
    let ground = &grounds[level.globals.normal_graphic_set as usize];
    for terrain in level.terrain.iter() {
        let terrain_info = &ground.ground.terrain_info[terrain.terrain_id as usize];
        let sprite = &ground.terrain_sprites[&terrain.terrain_id];
        draw(&sprite, 
            (terrain.x - rect.min_x) as i32, (terrain.y - rect.min_y) as i32,
            terrain_info.width as i32, terrain_info.height as i32,
            &mut rendered_level.bitmap, 
            width as i32, height as i32,
            terrain.do_not_overwrite_existing_terrain,
            terrain.is_upside_down,
            terrain.remove_terrain);
    }
    for object in level.objects.iter() {
        let object_info = &ground.ground.object_info[object.obj_id as usize];
        let sprite = &ground.object_sprites[&object.obj_id];
        draw(&sprite, 
            (object.x - rect.min_x) as i32, (object.y - rect.min_y) as i32,
            object_info.width as i32, object_info.height as i32, 
            &mut rendered_level.bitmap, 
            width as i32, height as i32, 
            false, false, false);
    }
    Ok(rendered_level)
}

fn main() -> io::Result<()> {
    use std::time::Instant;
    let now = Instant::now();
    let levels = load_all_levels()?;
    let grounds = load_all_grounds()?;
    let elapsed = now.elapsed();
    println!("Took: {:?}", elapsed); // 27ms optimised.

    // for (key, level) in &levels {
        let level = &levels[&0];
        let key = 0;
        let rendered = render_level(level, &grounds)?;
        let buf = u32_to_u8_slice(&rendered.bitmap);
        let filename = format!("output/levels/{} {}.png", key, level.name);
        image::save_buffer(filename, &buf, rendered.rect.width() as u32, rendered.rect.height() as u32, image::RGBA(8)).unwrap();
    // }

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
    // let main_raw: Vec<u8> = fs::read("data/MAIN.DAT")?;
    // let main_sections = decompressor::decompress(&main_raw)?;
    // let main = maindat::parse(&main_sections)?;
    // let image = main.main_menu.logo;
    // let buf = u32_to_u8_slice(&image.bitmap);
    // image::save_buffer("output/background.png", &buf, image.width as u32, image.height as u32, image::RGBA(8)).unwrap();

    Ok(())
}
