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
    object_sprites: HashMap<usize, Vec<u32>>, // TODO animate.
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
    for i in 0..5 { // TODO load by file search using for entry in fs::read_dir("data")? { if let Ok(entry) = entry {
        let ground = load_ground_and_sprites(i)?;
        all.push(ground);
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
    fn width(&self) -> isize {
        self.max_x - self.min_x
    }
    fn height(&self) -> isize {
        self.max_y - self.min_y
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

fn render_level(level: &level::Level, grounds: &[GroundCombined]) -> io::Result<RenderedLevel> {
    let rect = size_of_level(level, grounds);
    let width = rect.width();
    let height = rect.height();
    let pixels = (width * height) as usize;
    let mut rendered_level = RenderedLevel {
        bitmap: vec![0; pixels],
        rect: rect,
    };
    let ground = &grounds[level.globals.normal_graphic_set as usize];
    for terrain in level.terrain.iter() {
        let terrain_info = &ground.ground.terrain_info[terrain.terrain_id as usize];
    }
    for object in level.objects.iter() {
        let object = &ground.ground.object_info[object.obj_id as usize];
    }
    Ok(rendered_level)
}

fn main() -> io::Result<()> {
    let grounds = load_all_grounds()?;

    for i in 0..10 {
        extract_level(i, &grounds)?;
    }

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
