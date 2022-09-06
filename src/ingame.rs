use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::utils::HashMap;
use crate::{GameTextures, GameState, TEXTURE_SCALE};
use crate::lemmings::models::Game;
use crate::level_preview::LevelSelectionResource;
use crate::lemmings::level_renderer;
use crate::helpers::multi_scale;

pub struct InGamePlugin;

impl Plugin for InGamePlugin {
	fn build(&self, app: &mut App) {
		app.add_system_set(
			SystemSet::on_enter(GameState::InGame)
				.with_system(enter)
		);
		app.add_system_set(
			SystemSet::on_update(GameState::InGame)
				// .with_system(update)
		);
		app.add_system_set(
		    SystemSet::on_exit(GameState::InGame)
		        .with_system(exit)
		);
	}
}

#[derive(Component)]
struct InGameComponent; // Marker component so it can be despawned.

fn exit(
    mut commands: Commands,
    menu_components: Query<Entity, With<InGameComponent>>,
) {
    for e in menu_components.iter() {
        commands.entity(e).despawn_recursive();
    }
}

struct Slice {
    pub bitmap: Vec<u32>,
    pub width: usize,
    pub height: usize,
}

const SLICE_WIDTH: usize = 64; // N pixels in the bitmap, not display points or original lemmings pixels.

fn slice(image: &[u32], width: usize, height: usize) -> Vec<Slice> {
    let slices = Vec::<Slice>::with_capacity(width / SLICE_WIDTH + 1);
    let mut offset_x: usize = 0;
    while offset_x < width {
        let remaining_cols = width - offset_x;
        let this_width = std::cmp::max(SLICE_WIDTH, remaining_cols);
        let slice = Vec::<u32>::with_capacity(this_width * height);
        todo copy bits, make Slice, add it, etc.
        offset_x += SLICE_WIDTH;
    }
    slices
}
TODO make image handles

fn enter(
	mut commands: Commands,
	game_textures: Res<GameTextures>,
	level_selection: Res<LevelSelectionResource>,
	game: Res<Game>,
	mut images: ResMut<Assets<Image>>,
	windows: Res<Windows>,
) {
	if let Some(window) = windows.iter().next() {
		if let Some(level) = game.level_named(&level_selection.level_name) {
			let render = level_renderer::render(level, &game.grounds, &game.specials, true);
            let scaled = multi_scale(&render.image.bitmap, render.image.width, render.image.height, false);
            let slices = 

			commands
				.spawn_bundle(SpriteBundle{
                    sprite: Sprite { anchor: Anchor::BottomCenter, ..default() },
					texture: game_textures.skill_panel.clone(),
                    transform: Transform{
                        translation: Vec3::new(0., -window.height() / 2., 1.),
                        scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
                        ..default()
                    },        
                    ..default()
				})
				.insert(InGameComponent);
		}
	}
}
