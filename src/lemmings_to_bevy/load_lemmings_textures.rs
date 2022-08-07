use bevy::{prelude::*, render::render_resource::{Extent3d}, sprite::Rect};
use crate::lemmings::*;
use crate::lemmings::models::Animation;
use crate::xbrz;
use crate::{SCALE, SCALE_A, SCALE_B};
use crate::lemmings_to_bevy::image_doctor::*;

pub struct LoadLemmingsTexturesPlugin;

impl Plugin for LoadLemmingsTexturesPlugin {
	fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_lemmings_textures_startup);
	}
}

fn u32_to_rgba_u8(u32s: &[u32]) -> Vec<u8> {
    let mut u8s = Vec::<u8>::with_capacity(u32s.len() * 4);
    for u in u32s {
        u8s.push((u >> 24) as u8);
        u8s.push((u >> 16) as u8);
        u8s.push((u >> 8) as u8);
        u8s.push(*u as u8);
    }
    u8s
}

fn add_1_margin(image: &[u32], width: usize, height: usize) -> (Vec<u32>, usize, usize) {
    let margin_width = width + 2;
    let margin_height = height + 2;
    let margin_pixels = margin_width * margin_height;
    let mut image_with_margin = Vec::<u32>::with_capacity(margin_pixels);
    image_with_margin.resize(margin_pixels, 0);
    let mut in_offset: usize = 0;
    let mut out_offset: usize = margin_width + 1;
    for _y in 0..height {
        for _x in 0..width {
            image_with_margin[out_offset] = image[in_offset];
            out_offset += 1;
            in_offset += 1;
        }
        out_offset += 2;
    }
    (image_with_margin, margin_width, margin_height)
}

fn remove_scale_margin(image: &[u32], width: usize, height: usize) -> Vec<u32> {
    let smaller_width = width - 2 * SCALE;
    let smaller_height = height - 2 * SCALE;
    let smaller_pixels = smaller_width * smaller_height;
    let mut smaller_image = Vec::<u32>::with_capacity(smaller_pixels);
    smaller_image.resize(smaller_pixels, 0);
    let mut in_offset: usize = width * SCALE + SCALE;
    let mut out_offset: usize = 0;
    for _y in 0..smaller_height {
        for _x in 0..smaller_width {
            smaller_image[out_offset] = image[in_offset];
            out_offset += 1;
            in_offset += 1;
        }
        in_offset += 2 * SCALE;
    }
    smaller_image
}

// Multi-step scale-up.
// should_add_then_remove_margin removes artifacts from sprites (eg not things that are expected to tile) where they don't
// 'round off' near the edge properly.
fn multi_scale(image: &[u32], width: usize, height: usize, should_add_then_remove_margin: bool) -> Vec<u32> {
    if should_add_then_remove_margin {
        let (image_with_margin, margin_width, margin_height) = add_1_margin(image, width, height);
        let scaled = multi_scale(&image_with_margin, margin_width, margin_height, false);
        return remove_scale_margin(&scaled, margin_width * SCALE, margin_height * SCALE);
    }
    let bigger = xbrz::scale(SCALE_A as u8, image, width as u32, height as u32);
    xbrz::scale(SCALE_B as u8, &bigger, (width * SCALE_A) as u32, (height * SCALE_A) as u32)
}

// Figure out a neat way to layout the grid.
fn cols_rows_for_frames(frame_count: usize) -> (usize, usize) {
    let lenf = frame_count as f32;
    let cols = lenf.sqrt().round() as usize;
    let divides_perfectly = frame_count % cols == 0;
    let rows = if divides_perfectly { frame_count / cols } else { frame_count / cols + 1};
    (cols, rows)
}

fn make_atlas_from_animation(
    animation: &Animation,
    images: &mut ResMut<Assets<Image>>,
	texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    should_add_then_remove_margin: bool,
) -> Handle<TextureAtlas> {
    let (cols, rows) = cols_rows_for_frames(animation.frames.len());
    let scaled_width = animation.width * SCALE;
    let scaled_height = animation.height * SCALE;
    let atlas_width = (scaled_width + 1) * cols - 1; // 1px gap between each.
    let atlas_height = (scaled_height + 1) * rows - 1;
    let atlas_pixels = atlas_width * atlas_height;
    let mut atlas = Vec::<u32>::with_capacity(atlas_pixels);
    atlas.resize(atlas_pixels, 0);
    let mut col: usize = 0;
    let mut row: usize = 0;
    let mut sprite_rects = Vec::<Rect>::with_capacity(animation.frames.len());
    for small_frame in &animation.frames {
        let scaled_frame = multi_scale(small_frame, animation.width, animation.height, should_add_then_remove_margin);
        let start_atlas_x = col * (scaled_width + 1);
        let mut atlas_y = row * (scaled_height + 1);

        sprite_rects.push(Rect { 
            min: Vec2 { x: start_atlas_x as f32, y: atlas_y as f32 },
            max: Vec2 { x: (start_atlas_x + scaled_width) as f32, y: (atlas_y + scaled_height) as f32 } });

        for frame_y in 0..scaled_height {
            let mut atlas_x = start_atlas_x;
            for frame_x in 0..scaled_width {
                atlas[atlas_y * atlas_width + atlas_x] = scaled_frame[frame_y * scaled_width + frame_x];
                atlas_x += 1;
            }
            atlas_y += 1;
        }

        // Move to the next slot.
        col += 1;
        if col >= cols {
            col = 0;
            row += 1;
        }
    }
    // Convert it for bevy now.
    let u8_data = u32_to_rgba_u8(&atlas);
    let image = Image::new(Extent3d{width: atlas_width as u32, height: atlas_height as u32, depth_or_array_layers: 1},
        bevy::render::render_resource::TextureDimension::D2,
        u8_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb);
    let image_handle = images.add(image);
    let mut ta = TextureAtlas::new_empty(image_handle, Vec2::new(atlas_width as f32, atlas_height as f32));    
    for rect in sprite_rects {
        ta.add_texture(rect);
    }
    let ta_handle = texture_atlases.add(ta);
    ta_handle
}

fn make_image(
    image: &crate::lemmings::models::Image,
    images: &mut ResMut<Assets<Image>>,
    should_add_then_remove_margin: bool,
) -> Handle<Image> {
    let scaled = multi_scale(&image.bitmap, image.width, image.height, should_add_then_remove_margin);
    let u8_data = u32_to_rgba_u8(&scaled);
    let image = Image::new(Extent3d{width: (image.width * SCALE) as u32, height: (image.height * SCALE) as u32, depth_or_array_layers: 1},
        bevy::render::render_resource::TextureDimension::D2,
        u8_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb);
    let image_handle = images.add(image);
    image_handle
}

fn load_lemmings_textures_startup(
	mut commands: Commands,
	mut images: ResMut<Assets<Image>>,
	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // TODO multithread this!
    let games = loader::load().unwrap();
    let game = games.lemmings.unwrap();

    // Some of them need doctoring a bit.
    let background = doctor_clear_to_black(&game.main.main_menu.background);
    let f1 = doctor_f1(&game.main.main_menu.f1);
    let f2 = doctor_f2(&game.main.main_menu.f2);
    let f3 = doctor_f3(&game.main.main_menu.f3);
    let f4 = doctor_f4(&game.main.main_menu.f4);
    let level_rating = doctor_level_rating(&game.main.main_menu.level_rating);
    let exit_to_dos = doctor_exit_to_dos(&game.main.main_menu.exit_to_dos);
    let mayhem = doctor_skill(&game.main.main_menu.mayhem);
    let taxing = doctor_skill(&game.main.main_menu.taxing);
    let tricky = doctor_skill(&game.main.main_menu.tricky);
    let fun = doctor_skill(&game.main.main_menu.fun);

    {
    let data = crate::lemmings::png::png_data(game.main.main_menu.level_rating.width as u32, game.main.main_menu.level_rating.height as u32, &game.main.main_menu.level_rating.bitmap);
    std::fs::write("level_rating.original.png", &data).unwrap();
}
{
    let data = crate::lemmings::png::png_data(level_rating.width as u32, level_rating.height as u32, &level_rating.bitmap);
    std::fs::write("level_rating.doctored.png", &data).unwrap();
}
    
	let game_textures = GameTextures {
        // Menu:
        background: make_image(&background, &mut images, false),
        logo: make_image(&game.main.main_menu.logo, &mut images, true),
        f1: make_image(&f1, &mut images, true),
        f2: make_image(&f2, &mut images, true),
        f3: make_image(&f3, &mut images, true),
        f4: make_image(&f4, &mut images, true),
        level_rating: make_image(&level_rating, &mut images, true),
        exit_to_dos: make_image(&exit_to_dos, &mut images, true),
        music_note: make_image(&game.main.main_menu.music_note, &mut images, true),
        fx: make_image(&game.main.main_menu.fx, &mut images, true),
        blink1: make_atlas_from_animation(&game.main.main_menu.blink1, &mut images, &mut texture_atlases, false),
        blink2: make_atlas_from_animation(&game.main.main_menu.blink2, &mut images, &mut texture_atlases, false),
        blink3: make_atlas_from_animation(&game.main.main_menu.blink3, &mut images, &mut texture_atlases, false),
        blink4: make_atlas_from_animation(&game.main.main_menu.blink4, &mut images, &mut texture_atlases, false),
        blink5: make_atlas_from_animation(&game.main.main_menu.blink5, &mut images, &mut texture_atlases, false),
        blink6: make_atlas_from_animation(&game.main.main_menu.blink6, &mut images, &mut texture_atlases, false),
        blink7: make_atlas_from_animation(&game.main.main_menu.blink7, &mut images, &mut texture_atlases, false),
        left_scroller: make_atlas_from_animation(&game.main.main_menu.left_scroller, &mut images, &mut texture_atlases, true),
        right_scroller: make_atlas_from_animation(&game.main.main_menu.right_scroller, &mut images, &mut texture_atlases, true),
        reel: make_image(&game.main.main_menu.reel, &mut images, true),
        mayhem: make_image(&mayhem, &mut images, true),
        taxing: make_image(&taxing, &mut images, true),
        tricky: make_image(&tricky, &mut images, true),
        fun: make_image(&fun, &mut images, true),
    
        // Lemmings:
        walking_right: make_atlas_from_animation(&game.main.lemming_animations.walking_right, &mut images, &mut texture_atlases, true),
        jumping_right: make_image(&game.main.lemming_animations.jumping_right, &mut images, true),
        walking_left: make_atlas_from_animation(&game.main.lemming_animations.walking_left, &mut images, &mut texture_atlases, true),
        jumping_left: make_image(&game.main.lemming_animations.jumping_left, &mut images, true),
        digging: make_atlas_from_animation(&game.main.lemming_animations.digging, &mut images, &mut texture_atlases, true),
        climbing_right: make_atlas_from_animation(&game.main.lemming_animations.climbing_right, &mut images, &mut texture_atlases, true),
        climbing_left: make_atlas_from_animation(&game.main.lemming_animations.climbing_left, &mut images, &mut texture_atlases, true),
        drowning: make_atlas_from_animation(&game.main.lemming_animations.drowning, &mut images, &mut texture_atlases, true),
        post_climb_right: make_atlas_from_animation(&game.main.lemming_animations.post_climb_right, &mut images, &mut texture_atlases, true),
        post_climb_left: make_atlas_from_animation(&game.main.lemming_animations.post_climb_left, &mut images, &mut texture_atlases, true),
        brick_laying_right: make_atlas_from_animation(&game.main.lemming_animations.brick_laying_right, &mut images, &mut texture_atlases, true),
        brick_laying_left: make_atlas_from_animation(&game.main.lemming_animations.brick_laying_left, &mut images, &mut texture_atlases, true),
        bashing_right: make_atlas_from_animation(&game.main.lemming_animations.bashing_right, &mut images, &mut texture_atlases, true),
        bashing_left: make_atlas_from_animation(&game.main.lemming_animations.bashing_left, &mut images, &mut texture_atlases, true),
        mining_right: make_atlas_from_animation(&game.main.lemming_animations.mining_right, &mut images, &mut texture_atlases, true),
        mining_left: make_atlas_from_animation(&game.main.lemming_animations.mining_left, &mut images, &mut texture_atlases, true),
        falling_right: make_atlas_from_animation(&game.main.lemming_animations.falling_right, &mut images, &mut texture_atlases, true),
        falling_left: make_atlas_from_animation(&game.main.lemming_animations.falling_left, &mut images, &mut texture_atlases, true),
        pre_umbrella_right: make_atlas_from_animation(&game.main.lemming_animations.pre_umbrella_right, &mut images, &mut texture_atlases, true),
        umbrella_right: make_atlas_from_animation(&game.main.lemming_animations.umbrella_right, &mut images, &mut texture_atlases, true),
        pre_umbrella_left: make_atlas_from_animation(&game.main.lemming_animations.pre_umbrella_left, &mut images, &mut texture_atlases, true),
        umbrella_left: make_atlas_from_animation(&game.main.lemming_animations.umbrella_left, &mut images, &mut texture_atlases, true),
        splatting: make_atlas_from_animation(&game.main.lemming_animations.splatting, &mut images, &mut texture_atlases, true),
        exiting: make_atlas_from_animation(&game.main.lemming_animations.exiting, &mut images, &mut texture_atlases, true),
        fried: make_atlas_from_animation(&game.main.lemming_animations.fried, &mut images, &mut texture_atlases, true),
        blocking: make_atlas_from_animation(&game.main.lemming_animations.blocking, &mut images, &mut texture_atlases, true),
        shrugging_right: make_atlas_from_animation(&game.main.lemming_animations.shrugging_right, &mut images, &mut texture_atlases, true), // Builder running out of bricks.
        shrugging_left: make_atlas_from_animation(&game.main.lemming_animations.shrugging_left, &mut images, &mut texture_atlases, true),
        oh_no_ing: make_atlas_from_animation(&game.main.lemming_animations.oh_no_ing, &mut images, &mut texture_atlases, true),
        explosion: make_image(&game.main.lemming_animations.explosion, &mut images, true),
	};
	commands.insert_resource(game_textures);
}

pub struct GameTextures {
    // Menu:
    pub background: Handle<Image>,
    // pub background_width: usize,
    // pub background_height: usize,
    pub logo: Handle<Image>,
    pub f1: Handle<Image>,
    pub f2: Handle<Image>,
    pub f3: Handle<Image>,
    pub f4: Handle<Image>,
    pub level_rating: Handle<Image>,
    pub exit_to_dos: Handle<Image>,
    pub music_note: Handle<Image>,
    pub fx: Handle<Image>,
    pub blink1: Handle<TextureAtlas>,
    pub blink2: Handle<TextureAtlas>,
    pub blink3: Handle<TextureAtlas>,
    pub blink4: Handle<TextureAtlas>,
    pub blink5: Handle<TextureAtlas>,
    pub blink6: Handle<TextureAtlas>,
    pub blink7: Handle<TextureAtlas>,
    pub left_scroller: Handle<TextureAtlas>,
    pub right_scroller: Handle<TextureAtlas>,
    pub reel: Handle<Image>,
    pub mayhem: Handle<Image>,
    pub taxing: Handle<Image>,
    pub tricky: Handle<Image>,
    pub fun: Handle<Image>,
    //pub menu_font: MenuFont,

    // Lemmings:
    pub walking_right: Handle<TextureAtlas>,
    pub jumping_right: Handle<Image>, // Walking up a step 3-6px tall.
    pub walking_left: Handle<TextureAtlas>,
    pub jumping_left: Handle<Image>,
    pub digging: Handle<TextureAtlas>,
    pub climbing_right: Handle<TextureAtlas>,
    pub climbing_left: Handle<TextureAtlas>,
    pub drowning: Handle<TextureAtlas>,
    pub post_climb_right: Handle<TextureAtlas>,
    pub post_climb_left: Handle<TextureAtlas>,
    pub brick_laying_right: Handle<TextureAtlas>,
    pub brick_laying_left: Handle<TextureAtlas>,
    pub bashing_right: Handle<TextureAtlas>,
    pub bashing_left: Handle<TextureAtlas>,
    pub mining_right: Handle<TextureAtlas>,
    pub mining_left: Handle<TextureAtlas>,
    pub falling_right: Handle<TextureAtlas>,
    pub falling_left: Handle<TextureAtlas>,
    pub pre_umbrella_right: Handle<TextureAtlas>,
    pub umbrella_right: Handle<TextureAtlas>,
    pub pre_umbrella_left: Handle<TextureAtlas>,
    pub umbrella_left: Handle<TextureAtlas>,
    pub splatting: Handle<TextureAtlas>,
    pub exiting: Handle<TextureAtlas>,
    pub fried: Handle<TextureAtlas>,
    pub blocking: Handle<TextureAtlas>,
    pub shrugging_right: Handle<TextureAtlas>, // Builder running out of bricks.
    pub shrugging_left: Handle<TextureAtlas>,
    pub oh_no_ing: Handle<TextureAtlas>,
    pub explosion: Handle<Image>,
}

