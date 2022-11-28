use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use crate::{POINT_SIZE, TEXTURE_SCALE};
use crate::{GameTextures};

pub struct MouseCursorPlugin;

impl Plugin for MouseCursorPlugin {
	fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_mouse_cursor);
        app.add_system(mouse_motion_system);
	}
}

#[derive(Component)]
struct MouseCursorComponent;

fn spawn_mouse_cursor(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
) {
    commands.spawn_bundle(SpriteBundle {
        texture: game_textures.mouse_cursor.clone(),
        transform: Transform{
            translation: Vec3::new(0., 0., 999.),
            scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
            ..default()
        },        
        ..default()
    })
    .insert(MouseCursorComponent);
}

fn mouse_motion_system(
    windows: Res<Windows>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    // Is there some motion?
    let Some(event) = mouse_motion_events.iter().next() else { return };

    // Find the window.
    let Some(window) = windows.iter().next() else { return };

    if let Some(position) = window.cursor_position() {
        // Move and show it.
        // todo
    } else {
        // Off-window, so hide it.
    }
}
