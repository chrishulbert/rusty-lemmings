#![allow(dead_code)] // TODO disable once the app is mostly complete.

mod lemmings;
mod lemmings_to_bevy;
mod xbrz;
mod main_menu;
mod level_selection_menu;
mod menu_common;
mod fadeout;
mod level_preview;

use bevy::{
    prelude::*,
    window::PresentMode,
    // winit::WinitSettings, winit::UpdateMode,
};
use lemmings_to_bevy::load_lemmings_textures::GameTextures;
use bevy_inspector_egui::WorldInspectorPlugin;

// Tested by watching frame-by-frame youtube captures.
const FPS: f32 = 15.;
const FRAME_DURATION: f32 = 1. / FPS;

// 4k is 3840x2160
// 5K is 5120x2880
// Original game is 320x200
// Since it scrolls horizontally, i only care about height for scaling.
// 5k ratio is 14.4x high: could do 6x then 3x to get 18.
// 4k is 10.8x high
// Realistically: 6x then 2x to get 12: good enough for 4k.
// Or should we do 5x then 2x to get 10 and have a little margin for 4k?
// For a 720p window, we want an original pixels to be 720/200 = 3.6high. Divided by scale that is 0.3.
const SCALE: usize = 12; // Must be A*B.
const SCALE_A: usize = 6;
const SCALE_B: usize = 2;

const RES_W: usize = 1280;
const RES_H: usize = 720;
const ORIGINAL_GAME_H: usize = 200;
// I'm declaring an 'original game pixel' to be called a 'point'.
const POINT_SIZE: f32 = (RES_H as f32) / (ORIGINAL_GAME_H as f32); // How many bevy transform values to get one 'point' (pixel) in the original game.
const TEXTURE_SCALE: f32 = POINT_SIZE / (SCALE as f32);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    MainMenu,
    LevelSelectionMenu,
    Fading,
    LevelPreview,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}

fn startup(
    mut commands: Commands,
    // game_textures: Res<GameTextures>,
) {
    commands.spawn_bundle(Camera2dBundle::default());
    // commands
    //     .spawn_bundle(SpriteSheetBundle {
    //         texture_atlas: game_textures.mining_right.clone(),
    //         transform: Transform{
    //             translation: Vec3::new(0., 0., 1.),
    //             scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
    //             ..default()
    //         },        
    //         ..default()
    //     })
    //     .insert(AnimationTimer(Timer::from_seconds(FRAME_DURATION, true)));
    // commands
    //     .spawn_bundle(SpriteSheetBundle {
    //         texture_atlas: game_textures.blocking.clone(),
    //         transform: Transform{
    //             translation: Vec3::new(100., 0., 1.),
    //             scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
    //             ..default()
    //         },     
    //         ..default()
    //     })
    //     .insert(AnimationTimer(Timer::from_seconds(FRAME_DURATION, true)));
}

// #[derive(Component)]
// struct Person;

// #[derive(Component)]
// struct Name(String);

// fn add_people(mut commands: Commands) {
//     commands.spawn().insert(Person).insert(Name("Elaina Proctor".to_string()));
//     commands.spawn().insert(Person).insert(Name("Renzo Hume".to_string()));
//     commands.spawn().insert(Person).insert(Name("Zayna Nieves".to_string()));
// }

// struct GreetTimer(Timer);

// fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
//     if timer.0.tick(time.delta()).just_finished() {
//         for name in query.iter() {
//             println!("hello {}!", name.0);
//         }
//     }
// }

// pub struct HelloPlugin;

// impl Plugin for HelloPlugin {
//     fn build(&self, app: &mut App) {
//         app
//             .insert_resource(GreetTimer(Timer::from_seconds(2.0, true)))
//             .add_startup_system(add_people)
//             .add_system(greet_people);
//     }
// }

pub struct GameSelection(String); // Lemmings vs ohnomore vs christmas etc.

fn main() {
    // TODO think about how all the assets are centered, so that they can be blurry maybe?
    // Especially seems to affect even numbered ones? Or odd?
    App::new()
        // The following is a workaround for mouse lag, I hope it isn't necessary forever: https://github.com/bevyengine/bevy/issues/5778
        // .insert_resource(WinitSettings {
        //     focused_mode: UpdateMode::ReactiveLowPower { max_wait: std::time::Duration::from_millis(1000) },
        //     ..default()
        // })
        .add_state(GameState::MainMenu)
        .insert_resource(GameSelection("lemmings".to_string()))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            title: "Rusty Lemmings".to_string(),
            width: RES_W as f32,
            height: RES_H as f32,
            //resizable: false,
            present_mode: PresentMode::Fifo, // Battery-friendly vsync.
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(lemmings_to_bevy::load_lemmings_textures::LoadLemmingsTexturesPlugin)
        .add_plugin(fadeout::FadeoutPlugin)
        .add_plugin(main_menu::MainMenuPlugin)
        .add_plugin(level_selection_menu::LevelSelectionMenuPlugin)
        .add_plugin(level_preview::LevelPreviewPlugin)
        .add_startup_system(startup)
        .add_system(animate_sprite)
        .add_plugin(WorldInspectorPlugin::new())
        .run();
}
