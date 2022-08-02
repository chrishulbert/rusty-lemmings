use bevy::{prelude::*, render::render_resource::{Extent3d}, sprite::Rect};
use crate::lemmings::*;

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
fn scale_animation(a: &Animation) -> Animation {
    let scale_a: usize = 6;
    let scale_b: usize = 2;
    let mut big_frames = Vec::<Vec<u32>>::new();
    for frame in &a.frames {
        let bigger = xbrz::scale(scale_a as u8, frame, a.width as u32, a.height as u32);
        let biggest = xbrz::scale(scale_b as u8, &bigger, (a.width * scale_a) as u32, (a.height * scale_a) as u32);
        big_frames.push(biggest);
    }
    Animation{
        frames: big_frames,
        width: a.width * scale_a * scale_b,
        height: a.height * scale_a * scale_b,
    }
}

fn load_lemmings_textures_startup(
	mut commands: Commands,
	mut images: ResMut<Assets<Image>>,
	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let games = loader::load().unwrap();
    let game = games.lemmings.unwrap();
    let ml = &game.main.lemming_animations.mining_left;
    let u8_data = u32_to_rgba_u8(&ml.frames[0]);
    let image = Image::new(Extent3d{width: ml.width as u32, height: ml.height as u32, depth_or_array_layers: 1},
        bevy::render::render_resource::TextureDimension::D2,
        u8_data,
        bevy::render::render_resource::TextureFormat::Rgba8Unorm);
    let image_handle = images.add(image);

    let mut ta = TextureAtlas::new_empty(image_handle, Vec2::new(ml.width as f32, ml.height as f32));    
    ta.add_texture(Rect {
            min: Vec2::ZERO,
            max: Vec2::new(ml.width as f32, ml.height as f32),
        });
    let ta_handle = texture_atlases.add(ta);

	let game_textures = GameTextures {
		mining_right: ta_handle,
	};
	commands.insert_resource(game_textures);
}

pub struct GameTextures {
	pub mining_right: Handle<TextureAtlas>,
}

