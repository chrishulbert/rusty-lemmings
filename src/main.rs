use std::fs;
use std::io;
use std::mem;
use std::slice;

extern crate image;

mod decompressor;
mod ground;
mod sprites;

fn u32_to_u8_slice(original: &[u32]) -> &[u8] {
    let count = original.len() * mem::size_of::<u32>();
    let ptr = original.as_ptr() as *const u8;
    return unsafe { slice::from_raw_parts(ptr, count) };
}

fn main() -> io::Result<()> {
    let raw: Vec<u8> = fs::read("data/VGAGR0.DAT")?;
    let data = decompressor::decompress(&raw)?;
    println!("Number of sections: {:?}", data.len());
    for s in 0..data.len() {
        println!("Section #{}: {} bytes", s, data[s].len());
    }

    let ground_file: Vec<u8> = fs::read("data/GROUND0O.DAT")?;
    let ground = ground::parse(&ground_file)?;
    println!("VGA: {:x}", ground.palettes.vga_custom[0]);

    let palette = ground.palettes.as_abgr();

    for (i, terrain) in ground.terrain_info.iter().enumerate() {
        if terrain.is_valid() {
            // section[0]=terrain, section[1]=interactive objects.
            let sprite = sprites::extract(&data[0], terrain.width, terrain.height, terrain.image_loc, terrain.mask_loc, &palette);
            println!("{}: {:?}", i, terrain);

            let file = format!("terrain_{}.png", i);
            let buf = u32_to_u8_slice(&sprite);
            image::save_buffer(file, buf, terrain.width as u32, terrain.height as u32, image::RGBA(8)).unwrap();
        }
    }

    for (i, object) in ground.object_info.iter().enumerate() {
        if object.is_valid() {
            let sprite = sprites::extract(&data[1], object.width, object.height, object.animation_frames_base_loc, object.animation_frames_base_loc + object.mask_offset_from_image, &palette);
            println!("{}: {:?}", i, object);

            let file = format!("object_{}.png", i);
            let buf = u32_to_u8_slice(&sprite);
            image::save_buffer(file, buf, object.width as u32, object.height as u32, image::RGBA(8)).unwrap();
        }
    }

    Ok(())
}
