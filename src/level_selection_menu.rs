use bevy::prelude::*;
use crate::fadeout::create_fadeout;
use crate::{GameTextures, GameState, POINT_SIZE};
use crate::menu_common::{spawn_menu_background, text_size, spawn_text};
use crate::lemmings::levels_per_game_and_skill::names_per_game_and_skill;
use crate::level_preview::LevelSelectionResource;
use crate::lemmings::models::Game;

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
				.with_system(button_system)
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
    mut buttons: Query<(&Transform, &Children), With<LevelSelectionButton>>,
	mut letters: Query<&mut TextureAtlasSprite>,
) {
    if let Some(window) = windows.iter().next() {
        let position = window.cursor_position().unwrap_or(Vec2::NEG_ONE);
        let y = position.y - window.height() / 2.;
        for (transform, children) in &mut buttons {			
            let is_over = transform.translation.y - 16. * transform.scale.y < y &&
				y < transform.translation.y + 16. * transform.scale.y;
            let a: f32 = if is_over { if mouse_buttons.pressed(MouseButton::Left) { 0.5 } else { 0.8 } } else { 1. };
			for &child in children {
				if let Ok(mut letter) = letters.get_mut(child) {
					letter.color.set_a(a);
				}
			}
        }
    }
}

fn button_system(
    windows: Res<Windows>,
    mouse_buttons: Res<Input<MouseButton>>,
    buttons: Query<(&Transform, &LevelSelectionButton)>,
    game_textures: Res<GameTextures>,
    mut state: ResMut<State<GameState>>,
    mut level_selection: ResMut<LevelSelectionResource>,
    mut commands: Commands,
) {
    if mouse_buttons.just_released(MouseButton::Left) {
        if let Some(window) = windows.iter().next() {
            if let Some(position) = window.cursor_position() {
                let y = position.y - window.height() / 2.;
                let button_o = buttons.iter().find(|&b| {
                    b.0.translation.y - 16. * b.0.scale.y < y && y < b.0.translation.y + 16. * b.0.scale.y
                });
                if let Some(button) = button_o {
                    let lsb: &LevelSelectionButton = button.1;
					level_selection.level_name = lsb.level_name.to_string();
					level_selection.skill = lsb.skill;
					create_fadeout(&mut commands, GameState::LevelPreview, &game_textures, &mut state);
                }
            }
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

#[derive(Component)]
pub struct LevelSelectionButton{
	pub skill: isize,
	pub level_name: String,
}

fn spawn_level_button(parent: &mut ChildBuilder, game_textures: &Res<GameTextures>, name: &str, scale: f32, y: f32, skill: isize) {
	parent.spawn_bundle(SpatialBundle{
		transform: Transform {
			translation: Vec3::new(0., y, 2.),
			scale: Vec3::new(scale, scale, 1.),
			..default()
		},        
		..default()
	}).insert(LevelSelectionButton{
		skill,
		level_name: name.to_owned(),		
	}).with_children(|parent| {
		spawn_text(name, parent, game_textures);
	});
}

fn spawn_levels(
	mut commands: Commands,
	game_textures: Res<GameTextures>,
	skill_selection: Res<MainMenuSkillSelection>,
	game: Res<Game>,
) {
	commands
		.spawn_bundle(SpatialBundle::default())
		.insert(LevelSelectionMenuComponent)
		.with_children(|parent| {
			let names = names_per_game_and_skill(&game.id, skill_selection.0);
			let scale: f32 = if names.len() >= 16 { 0.5 } else { 1. };
			let padding: f32 = POINT_SIZE * 4. * scale;
			let size = text_size() * scale;
			let all_size: f32 = ((names.len() - 1) as f32) * (size + padding);
			let mut y: f32 = -all_size / 2.;
			for name in names {
				spawn_level_button(parent, &game_textures, &name, scale, y, skill_selection.0);
				y += size + padding;
			}
		});
}
