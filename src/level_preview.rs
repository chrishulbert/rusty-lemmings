use bevy::prelude::*;
use crate::{GameTextures, GameState, POINT_SIZE, TEXTURE_SCALE};
use crate::menu_common::{spawn_menu_background};

pub struct LevelPreviewPlugin;

impl Plugin for LevelPreviewPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(LevelSelectionResource::default());
		app.add_system_set(
			SystemSet::on_enter(GameState::LevelPreview)
				.with_system(spawn_background)
				// .with_system(spawn_levels)
		);
		app.add_system_set(
			SystemSet::on_update(GameState::LevelPreview)
				// .with_system(button_highlight_system)
		);
		app.add_system_set(
		    SystemSet::on_exit(GameState::LevelPreview)
		        .with_system(exit),
		);
	}
}

#[derive(Component)]
struct LevelPreviewComponent;

#[derive(Default)]
pub struct LevelSelectionResource {
	pub game_id: String,
	pub skill: isize,
	pub level_name: String,
}

fn exit(
    mut commands: Commands,
    menu_components: Query<Entity, With<LevelPreviewComponent>>,
) {
    for e in menu_components.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn spawn_background(
	mut commands: Commands,
	game_textures: Res<GameTextures>,
) {
	commands
		.spawn_bundle(SpatialBundle::default())
		.insert(LevelPreviewComponent)
		.with_children(|parent| {
			spawn_menu_background(parent, &game_textures);
		});
}
