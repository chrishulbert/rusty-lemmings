use bevy::{prelude::*, render::render_resource::{Extent3d}, sprite::Rect};
use crate::lemmings::*;
use crate::lemmings::models::Animation;
use crate::xbrz;

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

// 4k is 3840x2160
// 5K is 5120x2880
// Original game is 320x200
// Since it scrolls horizontally, i only care about height for scaling.
// 5k ratio is 14.4x high: could do 6x then 3x to get 18.
// 4k is 10.8x high
// Realistically: 6x then 2x to get 12: good enough for 4k.
// Or should we do 5x then 2x to get 10 and have a little margin for 4k?
const SCALE: usize = 12; // Must be A*B.
const SCALE_A: usize = 6;
const SCALE_B: usize = 2;

// Multi-step scale-up.
fn multi_scale(image: &[u32], width: usize, height: usize) -> Vec<u32> {
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
    mut images: ResMut<Assets<Image>>,
	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) -> Handle<TextureAtlas> {
    // TODO special case if only one treat as not an anim? Handle that in the parser?
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
        let scaled_frame = multi_scale(small_frame, animation.width, animation.height);
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
        bevy::render::render_resource::TextureFormat::Rgba8Unorm);
    let image_handle = images.add(image);
    let mut ta = TextureAtlas::new_empty(image_handle, Vec2::new(atlas_width as f32, atlas_height as f32));    
    for rect in sprite_rects {
        ta.add_texture(rect);
    }
    let ta_handle = texture_atlases.add(ta);
    ta_handle
}

fn load_lemmings_textures_startup(
	mut commands: Commands,
	images: ResMut<Assets<Image>>,
	texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let games = loader::load().unwrap();
    let game = games.lemmings.unwrap();
	let game_textures = GameTextures {
		mining_right: make_atlas_from_animation(&game.main.lemming_animations.mining_right, images, texture_atlases),
	};
	commands.insert_resource(game_textures);
}

pub struct GameTextures {
	pub mining_right: Handle<TextureAtlas>,
}

