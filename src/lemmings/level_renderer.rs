// This renders a level to a bitmap.

use std::cmp;
use std::io::Result;
use crate::lemmings::models::*;
use crate::lemmings::parsers::special;

const SPECIAL_LEFT_X: isize = 320;

#[derive(Debug, Copy, Clone)]
pub struct LevelSize {
    pub min_x: isize,
    pub max_x: isize,
}

impl LevelSize {
    pub fn width(&self) -> isize {
        self.max_x - self.min_x
    }
}

fn size_of_level(level: &Level, grounds: &GroundMap) -> LevelSize {
    if level.globals.extended_graphic_set != 0 {
        return LevelSize {
            min_x: SPECIAL_LEFT_X,
            max_x: SPECIAL_LEFT_X + special::WIDTH as isize,            
        }
    }

    let mut size = LevelSize {
        min_x: std::isize::MAX,
        max_x: std::isize::MIN,
    };
    let ground = &grounds[&(level.globals.normal_graphic_set as i32)];
    for terrain in level.terrain.iter() {
        let width = ground.ground.terrain_info[terrain.terrain_id as usize].width as isize;
        size.min_x = cmp::min(size.min_x, terrain.x);
        size.max_x = cmp::max(size.max_x, terrain.x + width);
    }
    size
}

pub struct RenderedLevel {
    pub image: Image,
    pub size: LevelSize,
}

const LEVEL_BACKGROUND: u32 = 0xff000000;
const LEVEL_HEIGHT: isize = 160;

fn draw(sprite: &[u32],
        sprite_width: isize, sprite_height: isize,
        x: isize, y: isize,
        canvas: &mut Vec<u32>, canvas_width: isize, canvas_height: isize,
        do_not_overwrite_existing_terrain: bool,
        is_upside_down: bool,
        remove_terrain: bool,
        must_have_terrain_underneath_to_be_visible: bool) {
    let mut canvas_offset = y * canvas_width + x;
    let canvas_stride = canvas_width - sprite_width;
    let mut sprite_offset: isize = if is_upside_down { (sprite_height - 1) * sprite_width } else { 0 };
    let sprite_stride: isize = if is_upside_down { -2 * sprite_width } else { 0 };
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

pub fn render(level: &Level, grounds: &GroundMap, specials: &SpecialMap) -> Result<RenderedLevel> {
    let size = size_of_level(level, grounds);
    let width = size.width();
    let height = LEVEL_HEIGHT;
    let pixels = width * height;
    let mut bitmap = vec![LEVEL_BACKGROUND; pixels as usize];
    let ground = &grounds[&(level.globals.normal_graphic_set as i32)];
    if level.globals.extended_graphic_set == 0 {
        for terrain in level.terrain.iter() {
            let sprite = &ground.terrain_sprites[&(terrain.terrain_id as i32)];
            draw(&sprite.bitmap,
                sprite.width as isize, sprite.height as isize,
                (terrain.x - size.min_x) as isize, terrain.y,
                &mut bitmap,
                width as isize, height as isize,
                terrain.do_not_overwrite_existing_terrain,
                terrain.is_upside_down,
                terrain.remove_terrain,
                false);
        }
    } else {
        let special = &specials[&(level.globals.extended_graphic_set as i32 - 1)];
        bitmap.copy_from_slice(&special.bitmap);
    }
    for object in level.objects.iter() {
        let sprite = &ground.object_sprites[&(object.obj_id as i32)];
        draw(&sprite.frames[0],
            sprite.width as isize, sprite.height as isize,
            object.x as isize - size.min_x, object.y as isize,
            &mut bitmap,
            width as isize, height as isize,
            object.modifier.is_do_not_overwrite_existing_terrain(),
            object.is_upside_down,
            false,
            object.modifier.is_must_have_terrain_underneath_to_be_visible());
    }
    let image = Image {
        bitmap,
        width: width as usize,
        height: height as usize,
    };
    let rendered_level = RenderedLevel {
        image: image,
        size: size,
    };
    Ok(rendered_level)
}
