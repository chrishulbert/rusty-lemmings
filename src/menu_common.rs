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
        parent.spawn_bundle(SpriteBundle {
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

pub fn button_highlight_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
