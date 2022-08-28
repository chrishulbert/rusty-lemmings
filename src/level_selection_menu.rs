use bevy::prelude::*;
use crate::{GameTextures, GameState, POINT_SIZE, GameSelection, TEXTURE_SCALE};
use crate::menu_common::{spawn_menu_background};
use crate::lemmings::levels_per_game_and_skill::names_per_game_and_skill;

#[derive(Component)]
struct LevelSelectionMenuComponent; // Marker component so the menu can be despawned.

pub struct MainMenuSkillSelection(pub isize);

pub struct LevelSelectionMenuPlugin;

impl Plugin for LevelSelectionMenuPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(MainMenuSkillSelection(1));
		app.add_system_set(
			SystemSet::on_enter(GameState::LevelSelectionMenu)
				// .with_system(enter)
				.with_system(spawn_background)
				.with_system(spawn_levels)
		);
		app.add_system_set(
			SystemSet::on_update(GameState::LevelSelectionMenu)
				// .with_system(update)
				.with_system(button_highlight_system)
		);
		app.add_system_set(
		    SystemSet::on_exit(GameState::LevelSelectionMenu)
		        .with_system(exit),
		);
	}
}

fn button_highlight_system(
    windows: Res<Windows>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut buttons: Query<(&mut Sprite, &Transform), With<LevelSelectionButton>>,
) {
    if let Some(window) = windows.iter().next() {
        let position = window.cursor_position().unwrap_or(Vec2::NEG_ONE);
        let x = position.x - window.width() / 2.;
        let y = position.y - window.height() / 2.;
        for (mut sprite, transform) in &mut buttons {
            let is_over = 
                transform.translation.x - 120. <= x && x <= transform.translation.x + 120. &&
                transform.translation.y - 61. <= y && y <= transform.translation.y + 61.;
            let a: f32 = if is_over { if mouse_buttons.pressed(MouseButton::Left) { 0.5 } else { 0.8 } } else { 1. };
            sprite.color.set_a(a);
        }
    }
}

fn exit(
    mut commands: Commands,
    menu_components: Query<Entity, With<LevelSelectionMenuComponent>>,
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
		.insert(LevelSelectionMenuComponent)
		.with_children(|parent| {
			spawn_menu_background(parent, &game_textures);
		});
}

fn text_size(scale: f32) -> f32 {
	16.0 * scale * POINT_SIZE / 2.
}

fn spawn_text(text: &str, parent: &mut ChildBuilder, game_textures: &Res<GameTextures>, scale_factor: f32, y: f32) {
	let texture_scale = scale_factor * TEXTURE_SCALE / 2.; // Logo is SVGA so halve it.
	let size = text_size(scale_factor);
	let scale = Vec3::new(texture_scale, texture_scale, 1.);
	let mut x: f32 = -(text.len() as f32) / 2. * size;
	for c in text.chars() {
		let a = c as u32;
		if 33 <= a && a <= 126 { // Menu font is '!'(33) - '~'(126)
			let index = (a - 33) as usize;
			parent.spawn_bundle(SpriteSheetBundle {
				texture_atlas: game_textures.menu_font.clone(),
				sprite: TextureAtlasSprite{index, ..default()},
				transform: Transform {
					scale,
					translation: Vec3::new(x, y, 2.),
					..default()
				},        
				..default()
			});
		}
		x += size;
	}
}

#[derive(Component)]
pub struct LevelSelectionButton{
	pub game_id: String,
	pub skill: isize,
	pub level_name: String,
}

fn spawn_level_button(parent: &mut ChildBuilder, game_textures: &Res<GameTextures>, name: &str, scale: f32, y: f32, game_id: &str, skill: isize) {
	parent.spawn_bundle(SpatialBundle{
		..default()
	}).insert(LevelSelectionButton{
		game_id: game_id.to_owned(),
		skill,
		level_name: name.to_owned(),		
	}).with_children(|parent| {
		spawn_text(name, parent, game_textures, scale, y);
	});
}

fn spawn_levels(
	mut commands: Commands,
	game_textures: Res<GameTextures>,
	skill_selection: Res<MainMenuSkillSelection>,
	game_selection: Res<GameSelection>,
) {
	commands
		.spawn_bundle(SpatialBundle::default())
		.insert(LevelSelectionMenuComponent)
		.with_children(|parent| {
			let names = names_per_game_and_skill(&game_selection.0, skill_selection.0);
			let scale: f32 = if names.len() >= 16 { 0.5 } else { 1. };
			let size = text_size(scale);
			let mut y: f32 = -(names.len() as f32) / 2. * size;
			for name in names {
				spawn_level_button(parent, &game_textures, &name, scale, y, &game_selection.0, skill_selection.0);
				y += size
			}
		});
}
