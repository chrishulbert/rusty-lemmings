use bevy::{prelude::*, render::render_resource::{Extent3d}};
use crate::lemmings::*;

pub struct LoadLemmingsTexturesPlugin;

impl Plugin for LoadLemmingsTexturesPlugin {
	fn build(&self, app: &mut App) {
		app.add_startup_system(load_lemmings_textures_startup);
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
        bevy::render::render_resource::TextureFormat::Rgba8Uint);
    let image_handle = images.add(image);

    let ta = TextureAtlas::new_empty(image_handle, Vec2::new(ml.width as f32, ml.height as f32));    
    // ta.add_texture(Rect {
    //         min: rect_min,
    //         max: Vec2::new(rect_min.x + tile_size.x, rect_min.y + tile_size.y),
    //     });
    let ta_handle = texture_atlases.add(ta);

	let game_textures = GameTextures {
		mining_right: ta_handle,
	};
	commands.insert_resource(game_textures);
}

struct GameTextures {
	mining_right: Handle<TextureAtlas>,
}

