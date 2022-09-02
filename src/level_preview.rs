use bevy::prelude::*;
use bevy::sprite::Anchor;
use crate::{GameTextures, GameState};
use crate::menu_common::{spawn_menu_background, text_size, spawn_text};
use crate::lemmings::level_renderer;
use crate::lemmings::models::Game;

pub struct LevelPreviewPlugin;

impl Plugin for LevelPreviewPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(LevelSelectionResource::default());
		app.add_system_set(
			SystemSet::on_enter(GameState::LevelPreview)
				.with_system(spawn_background)
				.with_system(spawn_preview)
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

// fn button_system(
//     windows: Res<Windows>,
//     mouse_buttons: Res<Input<MouseButton>>,
//     buttons: Query<(&Transform, &LevelSelectionButton)>,
//     game_textures: Res<GameTextures>,
//     mut state: ResMut<State<GameState>>,
//     mut level_selection: ResMut<LevelSelectionResource>,
//     mut commands: Commands,
// ) {
//     if mouse_buttons.just_released(MouseButton::Left) {
//         if let Some(window) = windows.iter().next() {

fn spawn_background(
	mut commands: Commands,
	game_textures: Res<GameTextures>,
	windows: Res<Windows>,
) {
	commands
		.spawn_bundle(SpatialBundle::default())
		.insert(LevelPreviewComponent)
		.with_children(|parent| {
			spawn_menu_background(parent, &game_textures);

			// Top black area: 78/350 of screen size.
			if let Some(window) = windows.iter().next() {
				let mini_map_background_height = (window.height() * 78. / 350.).ceil();
				parent.spawn_bundle(SpriteBundle {
					texture: game_textures.white.clone(),
					sprite: Sprite { 
						color: Color::rgba(0., 0., 0., 1.), 
						custom_size: Some(Vec2::new(9999., 9999.)),
						anchor: Anchor::BottomCenter,
						..default() 
					},
					transform: Transform {
						translation: Vec3::new(0., window.height() / 2. - mini_map_background_height, 1.),
						..default()
					},        
					..default()
				});

							// TODO actual minimap 39/350 high.

			}
		});
}

fn spawn_preview(
	mut commands: Commands,
	game_textures: Res<GameTextures>,
	level_selection: Res<LevelSelectionResource>,
	game: Res<Game>,
	mut images: ResMut<Assets<Image>>,
) {
	let text: Vec<String> = vec![
		"Level 1".to_string(),
		level_selection.level_name.to_string(),
		"".to_string(),
		"Number of Lemmings: X".to_string(),
		"To be saved: 10%".to_string(),
		"Release rate: 50".to_string(),
		"Time: 5 minutes".to_string(),
		"Rating: Fun".to_string(),
		"".to_string(),
		"Press mouse button to continue".to_string(),
	];
	let size = text_size();
	let gap = (size / 2.).round();
	let text_lines = text.len();
	let all_height = (size + gap) * ((text_lines - 1) as f32); // From center of topmost to center of bottom-most.

	commands
		.spawn_bundle(SpatialBundle::default())
		.insert(LevelPreviewComponent)
		.with_children(|parent| {
			for (i, t) in text.iter().enumerate() {
				parent.spawn_bundle(SpatialBundle{
					transform: Transform::from_xyz(0., all_height / 2. - ((i as f32) * (size + gap)), 2.),
					..default()
				}).with_children(|parent| {
					spawn_text(t, parent, &game_textures);
				});
			}
		});

	if let Some(level) = game.level_named(&level_selection.level_name) {
		// let render = level_renderer::render(level, &game.grounds, &game.specials, true);
		// render.image
		// let handle = images.

		// let white_data: Vec<u8> = vec![0xff, 0xff, 0xff, 0xff];
		// let white_image = Image::new(Extent3d{width: 1, height: 1, depth_or_array_layers: 1},
		// 	bevy::render::render_resource::TextureDimension::D2,
		// 	white_data,
		// 	bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb);
		// let white = images.add(white_image);
	}
}