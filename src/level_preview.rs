use bevy::prelude::*;
use bevy::sprite::Anchor;
use crate::fadeout::*;
use crate::{GameTextures, GameState};
use crate::menu_common::{spawn_menu_background, text_size, spawn_text};
use crate::lemmings::level_renderer;
use crate::lemmings::models::Game;
use crate::helpers::make_image_unscaled;

pub struct LevelPreviewPlugin;

impl Plugin for LevelPreviewPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(LevelSelectionResource::default());
		app.add_systems((
            spawn_background,
            spawn_preview,
        ).in_schedule(OnEnter(GameState::LevelPreview)));
        app.add_systems((
            button_system.run_if(screen_fade_is_not_transitioning),
        ).in_set(OnUpdate(GameState::LevelPreview)));
        app.add_systems((
            exit,
        ).in_schedule(OnExit(GameState::LevelPreview)));
	}
}

#[derive(Component)]
struct LevelPreviewComponent;

#[derive(Resource)]
pub struct LevelSelectionResource {
	pub skill: isize,
	pub level_name: String,
}
impl Default for LevelSelectionResource {
	fn default() -> Self {
		Self {
			skill: 0,
			level_name: "Just dig!".to_string(),
		}		
	}
}

fn exit(
    mut commands: Commands,
    menu_components: Query<Entity, With<LevelPreviewComponent>>,
) {
    for e in menu_components.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn button_system(
	mut commands: Commands,
    is_transitioning: ResMut<ScreenFadeIsTransitioning>,
	game_textures: Res<GameTextures>,
    mouse_buttons: Res<Input<MouseButton>>,
) {
    if mouse_buttons.just_released(MouseButton::Left) {
		create_fadeout(&mut commands, GameState::InGame, &game_textures, is_transitioning);
	}
}

fn spawn_background(
	mut commands: Commands,
	game_textures: Res<GameTextures>,
) {
	commands
		.spawn(SpatialBundle::default())
		.insert(LevelPreviewComponent)
		.with_children(|parent| {
			spawn_menu_background(parent, &game_textures);
		});
}

fn spawn_preview(
	mut commands: Commands,
	game_textures: Res<GameTextures>,
	level_selection: Res<LevelSelectionResource>,
	game: Res<Game>,
	mut images: ResMut<Assets<Image>>,
	windows: Query<&Window>,
) {
	if let Some(window) = windows.iter().next() {
		if let Some(level) = game.level_named(&level_selection.level_name) {
			// Top black area: 78/350 of screen size.
			let mini_map_background_height = (window.height() * 78. / 350.).ceil();
			
			// Text.
			let text: Vec<String> = vec![
				level_selection.level_name.to_string(),
				"".to_string(),
				format!("Number of Lemmings: {}", level.globals.num_of_lemmings),
				format!("To be saved: {}", level.globals.num_to_rescue),
				format!("Release rate: {}", level.globals.release_rate),
				format!("Time: {} minutes", level.globals.time_limit),
				"".to_string(),
				"Press mouse button to continue".to_string(),
			];
			let size = text_size();
			let gap = (size / 2.).round();
			let text_lines = text.len();
			let all_height = (size + gap) * ((text_lines - 1) as f32); // From center of topmost to center of bottom-most.
			let text_center_y_offset = -mini_map_background_height / 2.; // Center it in the remaining space under the black bar.
			commands
				.spawn(SpatialBundle::default())
				.insert(LevelPreviewComponent)
				.with_children(|parent| {
					for (i, t) in text.iter().enumerate() {
						parent.spawn(SpatialBundle{
							transform: Transform::from_xyz(0., text_center_y_offset + all_height / 2. - ((i as f32) * (size + gap)), 2.),
							..default()
						}).with_children(|parent| {
							spawn_text(t, parent, &game_textures);
						});
					}
				});

			// Black bar.
			commands
				.spawn(SpriteBundle {
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
				})
				.insert(LevelPreviewComponent);

			// Minimap.
			let mini_map_height = (window.height() * 39. / 350.).ceil();
			let render = level_renderer::render(level, &game.grounds, &game.specials, true);
			let level_texture = make_image_unscaled(&render.image, &mut images);
			let scale_width: f32 = (render.image.width as f32) / (render.image.height as f32) * mini_map_height;
			commands
				.spawn(SpriteBundle{
					sprite: Sprite { custom_size: Some(Vec2::new(scale_width, mini_map_height)), ..default() },
					transform: Transform::from_xyz(0., window.height() / 2. - mini_map_background_height / 2., 2.),
					texture: level_texture,
					..default()
				})
				.insert(LevelPreviewComponent);
		}
	}
}
