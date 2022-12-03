use bevy::prelude::*;
use crate::GameTextures;
use crate::{POINT_SIZE, TEXTURE_SCALE};

pub const NORMAL_BUTTON: Color = Color::NONE;
pub const HOVERED_BUTTON: Color = Color::rgba(0., 0., 0., 0.5);
pub const PRESSED_BUTTON: Color = Color::rgba(1., 1., 1., 0.02);

pub fn spawn_menu_background(
    parent: &mut ChildBuilder,
    game_textures: &Res<GameTextures>,
) {
    const BG_WIDTH: f32 = 320.; // Texture size in original game pixels (points).
    const BG_HEIGHT: f32 = 104.;
    fn spawn(parent: &mut ChildBuilder, game_textures: &Res<GameTextures>, x: f32, y: f32) {
        parent.spawn(SpriteBundle {
            texture: game_textures.background.clone(),
            transform: Transform{
                translation: Vec3::new(x * POINT_SIZE, y * POINT_SIZE, 0.),
                scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
                ..default()
            },        
            ..default()
        });
    }
    spawn(parent, &game_textures, BG_WIDTH, BG_HEIGHT);
    spawn(parent, &game_textures, 0., BG_HEIGHT);
    spawn(parent, &game_textures, -BG_WIDTH, BG_HEIGHT);
    spawn(parent, &game_textures, BG_WIDTH, 0.);
    spawn(parent, &game_textures, 0., 0.);
    spawn(parent, &game_textures, -BG_WIDTH, 0.);
    spawn(parent, &game_textures, BG_WIDTH, -BG_HEIGHT);
    spawn(parent, &game_textures, 0., -BG_HEIGHT);
    spawn(parent, &game_textures, -BG_WIDTH, -BG_HEIGHT);    
}

pub fn text_size() -> f32 {
	16.0 * POINT_SIZE / 2.
}

pub fn spawn_text(text: &str, parent: &mut ChildBuilder, game_textures: &Res<GameTextures>) {
	let texture_scale = TEXTURE_SCALE / 2.; // Logo is SVGA so halve it.
	let size = text_size();
	let scale = Vec3::new(texture_scale, texture_scale, 1.);
	let mut x: f32 = -((text.len() as f32) - 1.) / 2. * size;
	for c in text.chars() {
		let a = c as u32;
		if 33 <= a && a <= 126 { // Menu font is '!'(33) - '~'(126)
			let index = (a - 33) as usize;
			parent.spawn(SpriteSheetBundle {
				texture_atlas: game_textures.menu_font.clone(),
				sprite: TextureAtlasSprite{index, ..default()},
				transform: Transform {
					scale,
					translation: Vec3::new(x, 0., 3.),
					..default()
				},        
				..default()
			});
		}
		x += size;
	}
}