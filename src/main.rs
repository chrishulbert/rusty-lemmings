use std::fs;
use std::io;
use std::mem;
use std::slice;

extern crate image;

mod decompressor;
mod ground;
mod sprites;
mod level;
mod special;
mod maindat;

fn u32_to_u8_slice(original: &[u32]) -> &[u8] {
    let count = original.len() * mem::size_of::<u32>();
    let ptr = original.as_ptr() as *const u8;
    return unsafe { slice::from_raw_parts(ptr, count) };
}

fn extract(index: u8) -> io::Result<()> {
    println!("Extracting {}", index);
    fs::create_dir_all(format!("output/{}", index))?;

    let data_file: Vec<u8> = fs::read(format!("data/VGAGR{}.DAT", index))?;
    let data = decompressor::decompress(&data_file)?;
    println!("Number of sections: {:?}", data.len());
    for s in 0..data.len() {
        println!("Section #{}: {} bytes", s, data[s].len());
    }

    let ground_file: Vec<u8> = fs::read(format!("data/GROUND{}O.DAT", index))?;
    let ground = ground::parse(&ground_file)?;
    println!("VGA: {:x}", ground.palettes.vga_custom[0]);

    let palette = ground.palettes.as_abgr();

    for (i, terrain) in ground.terrain_info.iter().enumerate() {
        if terrain.is_valid() {
            // section[0]=terrain, section[1]=interactive objects.
            let sprite = sprites::extract(&data[0], terrain.width, terrain.height, terrain.image_loc, terrain.mask_loc, &palette);
            println!("{}: {:?}", i, terrain);

            let file = format!("output/{}/terrain_{}.png", index, i);
            let buf = u32_to_u8_slice(&sprite);
            image::save_buffer(file, buf, terrain.width as u32, terrain.height as u32, image::RGBA(8)).unwrap();
        }
    }

    for (i, object) in ground.object_info.iter().enumerate() {
        if object.is_valid() {
            let sprite = sprites::extract(&data[1], object.width, object.height, object.animation_frames_base_loc, object.animation_frames_base_loc + object.mask_offset_from_image, &palette);
            println!("{}: {:?}", i, object);

            let file = format!("output/{}/object_{}.png", index, i);
            let buf = u32_to_u8_slice(&sprite);
            image::save_buffer(file, buf, object.width as u32, object.height as u32, image::RGBA(8)).unwrap();
        }
    }

    Ok(())
}

fn extract_level(index: isize) -> io::Result<()> {
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

fn main() -> io::Result<()> {
    for i in 0..5 {
        extract(i)?;
    }

    for i in 0..10 {
        extract_level(i)?;
    }

    for i in 0..4 {
        extract_special(i)?;
    }

    // Main.dat
    let main_raw: Vec<u8> = fs::read("data/MAIN.DAT")?;
    let main_sections = decompressor::decompress(&main_raw)?;
    let main = maindat::parse(&main_sections)?;
    let image = main.main_menu.logo;
    let buf = u32_to_u8_slice(&image.bitmap);
    image::save_buffer("output/background.png", &buf, image.width as u32, image.height as u32, image::RGBA(8)).unwrap();

    Ok(())
}
