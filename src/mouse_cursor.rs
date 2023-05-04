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
        app.insert_resource(MouseCursorIsSelectorSquare(false));
        app.add_event::<MouseCursorShouldBecomeSelectorEvent>();
	}
}

#[derive(Component)]
pub struct MouseCursorComponent;

#[derive(Resource)]
pub struct MouseCursorIsSelectorSquare(bool);

pub struct MouseCursorShouldBecomeSelectorEvent(pub bool);

fn spawn_mouse_cursor(
    mut commands: Commands,
    mut windows: Query<&mut Window>,
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

    // https://blog.rust-lang.org/2022/11/03/Rust-1.65.0.html#let-else-statements
    let Some(mut window) = windows.iter_mut().next() else { return };
    window.cursor.visible = false;
}

/// Add this to your game state after sending MouseCursorShouldBecomeSelectorEvent.
pub fn update_mouse_cursor_style_system(
    mut should_become_selector: EventReader<MouseCursorShouldBecomeSelectorEvent>,
    mut is_selector_square: ResMut<MouseCursorIsSelectorSquare>, 
    mut cursor_query: Query<&mut Handle<Image>, With<MouseCursorComponent>>,
    game_textures: Res<GameTextures>,
) {
    let Some(should_be_selector) = should_become_selector.iter().next() else { return };
    if should_be_selector.0 == is_selector_square.0 { return }; // To save time, don't change unless necessary.
    for mut handle in cursor_query.iter_mut() {
        if should_be_selector.0 {
            *handle = game_textures.mouse_cursor_hovering.clone();
        } else {
            *handle = game_textures.mouse_cursor.clone();
        }
    }
    is_selector_square.0 = should_be_selector.0;
}

// Call this when exiting your game state.
pub fn reset_mouse_cursor_system(
    mut is_selector_square: ResMut<MouseCursorIsSelectorSquare>, 
    game_textures: Res<GameTextures>,
    mut cursor_query: Query<&mut Handle<Image>, With<MouseCursorComponent>>,
) {
    for mut handle in cursor_query.iter_mut() {
        *handle = game_textures.mouse_cursor.clone();
    }
    is_selector_square.0 = false;
}

fn mouse_motion_system(
    windows: Query<&Window>,
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
