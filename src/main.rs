use std::fs;
use std::io;

mod decompressor;
mod ground;
mod sprites;

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

    let palette = ground.palettes.as_rgba();

    for (i, terrain) in ground.terrain_info.iter().enumerate() {
        if terrain.is_valid() {
            // section[0]=terrain, section[1]=interactive objects.
            let sprite = sprites::extract(&data[0], terrain.width, terrain.height, terrain.image_loc, terrain.mask_loc, &palette);
            println!("{}: {:?}; {:?}", i, terrain, sprite);
        }
    }

    Ok(())
}
