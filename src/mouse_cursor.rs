use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use crate::TEXTURE_SCALE;
use crate::GameTextures;

const MOUSE_Z: f32 = 999.; // On top of all.

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
    mut windows: ResMut<Windows>,
    game_textures: Res<GameTextures>,
) {
    commands.spawn(SpriteBundle {
        texture: game_textures.mouse_cursor.clone(),
        transform: Transform{
            translation: Vec3::new(99999., 99999., MOUSE_Z),
            scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
            ..default()
        },        
        ..default()
    })
    .insert(MouseCursorComponent); // TODO would it be more efficient to store the id instead?

    windows.primary_mut().set_cursor_visibility(false);
}

fn mouse_motion_system(
    windows: Res<Windows>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_cursor_component_query: Query<&mut Transform, With<MouseCursorComponent>>,
) {
    // Is there some motion? Otherwise dont bother with the remainder, as an optimisation.
    let Some(_) = mouse_motion_events.iter().next() else { return };

    // Find the window.
    let Some(window) = windows.iter().next() else { return };

    for mut transform in &mut mouse_cursor_component_query {
        if let Some(position) = window.cursor_position() {            
            // Move and show it.
            transform.translation = Vec3::new(position.x - window.width() / 2., position.y - window.height() / 2., MOUSE_Z);
        } else {
            // Off-window, so hide it.
            transform.translation = Vec3::new(99999., 99999., MOUSE_Z);
        }
    }
}
