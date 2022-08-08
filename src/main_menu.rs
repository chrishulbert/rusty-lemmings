use bevy::prelude::*;
use crate::GameTextures;
use crate::GameState;
use crate::{POINT_SIZE, TEXTURE_SCALE, FRAME_DURATION};

pub struct MainMenuPlugin;

#[derive(Component)]
struct BlinkAnimationTimer {
    timer: Timer,
    index: isize, // Set to negative if you want a delay before the blink.
    dwell: isize, // Dwell time in frames after a blink.
}

fn animate_blinking_sprites(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut BlinkAnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in &mut query {
        timer.timer.tick(time.delta());
        if timer.timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            let texture_frames = texture_atlas.textures.len() as isize;

            // Advance.
            timer.index += 1;
            let total_frames = texture_frames + timer.dwell;
            if timer.index > total_frames {
                timer.index = 0;
            }

            // Apply it. If it's in the initial delay or the dwell time, just show 0.
            if 0 <= timer.index && timer.index < texture_frames {
                sprite.index = timer.index as usize;
            } else {
                sprite.index = 0;
            }
        }
    }
}

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::MainMenu)
                .with_system(enter)
                .with_system(spawn_menu_logo)       
                .with_system(spawn_menu_background),
        )
        .add_system_set(
            SystemSet::on_update(GameState::MainMenu)
                .with_system(update)
                .with_system(animate_blinking_sprites),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::MainMenu)
                .with_system(exit),
        );
	}
}

fn enter(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
) {

}

fn update(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
) {

}

fn exit(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
) {

}

fn spawn_menu_background(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
) {
    const BG_WIDTH: f32 = 320.; // Texture size in original game pixels (points).
    const BG_HEIGHT: f32 = 104.;
    fn spawn(commands: &mut Commands, game_textures: &Res<GameTextures>, x: f32, y: f32) {
        commands
        .spawn_bundle(SpriteBundle {
            texture: game_textures.background.clone(),
            transform: Transform{
                translation: Vec3::new(x * POINT_SIZE, y * POINT_SIZE, 0.),
                scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
                ..default()
            },        
            ..default()
        });
    }
    spawn(&mut commands, &game_textures, BG_WIDTH, BG_HEIGHT);
    spawn(&mut commands, &game_textures, 0., BG_HEIGHT);
    spawn(&mut commands, &game_textures, -BG_WIDTH, BG_HEIGHT);
    spawn(&mut commands, &game_textures, BG_WIDTH, 0.);
    spawn(&mut commands, &game_textures, 0., 0.);
    spawn(&mut commands, &game_textures, -BG_WIDTH, 0.);
    spawn(&mut commands, &game_textures, BG_WIDTH, -BG_HEIGHT);
    spawn(&mut commands, &game_textures, 0., -BG_HEIGHT);
    spawn(&mut commands, &game_textures, -BG_WIDTH, -BG_HEIGHT);
}

fn spawn_menu_logo(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
) {
    let scale = Vec3::new(TEXTURE_SCALE / 2., TEXTURE_SCALE / 2., 1.); // Logo is svga, so halve everything.
    const Y: f32 = 52.; // Make it overlap the background-tile-seam.
    commands
        .spawn_bundle(SpriteBundle {
            texture: game_textures.logo.clone(),
            transform: Transform{
                translation: Vec3::new(0., Y * POINT_SIZE, 1.), 
                scale,
                ..default()
            },        
            ..default()
        });

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_textures.blink1.clone(),
            transform: Transform{
                translation: Vec3::new(-138. * POINT_SIZE, (Y + 1.) * POINT_SIZE, 2.),
                scale,
                ..default()
            },        
            ..default()
        })
        .insert(BlinkAnimationTimer{
            timer: Timer::from_seconds(FRAME_DURATION, true),
            index: -15,
            dwell: 30,
        });

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_textures.blink2.clone(),
            transform: Transform{
                translation: Vec3::new(-26. * POINT_SIZE, (Y + 2.) * POINT_SIZE, 2.),
                scale,
                ..default()
            },        
            ..default()
        })
        .insert(BlinkAnimationTimer{
            timer: Timer::from_seconds(FRAME_DURATION, true),
            index: -22,
            dwell: 45,
        });

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_textures.blink3.clone(),
            transform: Transform{
                translation: Vec3::new(94. * POINT_SIZE, (Y + 5.5) * POINT_SIZE, 2.),
                scale,
                ..default()
            },        
            ..default()
        })
        .insert(BlinkAnimationTimer{
            timer: Timer::from_seconds(FRAME_DURATION, true),
            index: -30,
            dwell: 37,
        });
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_textures.blink3.clone(),
            transform: Transform{
                translation: Vec3::new(94. * POINT_SIZE, (Y + 5.5) * POINT_SIZE, 2.),
                scale,
                ..default()
            },        
            ..default()
        })
        .insert(BlinkAnimationTimer{
            timer: Timer::from_seconds(FRAME_DURATION, true),
            index: -30,
            dwell: 37,
        });

        commands
        .spawn_bundle(SpriteBundle {
            texture: game_textures.f1.clone(),
            transform: Transform{
                translation: Vec3::new(-100. * POINT_SIZE, 0., 2.),
                scale,
                ..default()
            },        
            ..default()
        });
        commands
        .spawn_bundle(SpriteBundle {
            texture: game_textures.f2.clone(),
            transform: Transform{
                translation: Vec3::new(-33. * POINT_SIZE, 0., 2.),
                scale,
                ..default()
            },        
            ..default()
        });
        commands
        .spawn_bundle(SpriteBundle {
            texture: game_textures.f3.clone(),
            transform: Transform{
                translation: Vec3::new(33. * POINT_SIZE, 0., 2.),
                scale,
                ..default()
            },        
            ..default()
        });
        commands
        .spawn_bundle(SpriteBundle {
            texture: game_textures.level_rating.clone(),
            transform: Transform{
                translation: Vec3::new(100. * POINT_SIZE, 0., 2.),
                scale,
                ..default()
            },        
            ..default()
        });
        commands
        .spawn_bundle(SpriteBundle {
            texture: game_textures.exit_to_dos.clone(),
            transform: Transform{
                translation: Vec3::new(-33. * POINT_SIZE, -50. * POINT_SIZE, 2.),
                scale,
                ..default()
            },        
            ..default()
        });
        commands
        .spawn_bundle(SpriteBundle {
            texture: game_textures.f4.clone(),
            transform: Transform{
                translation: Vec3::new(33. * POINT_SIZE, -50. * POINT_SIZE, 2.),
                scale,
                ..default()
            },        
            ..default()
        });
        // commands
        // .spawn_bundle(SpriteBundle {
        //     texture: game_textures.fun.clone(),
        //     transform: Transform{
        //         translation: Vec3::new(-100. * POINT_SIZE, -5. * POINT_SIZE, 3.),
        //         scale,
        //         ..default()
        //     },        
        //     ..default()
        // });
}
