#![allow(dead_code)] // TODO disable once the app is mostly complete.

mod lemmings;
mod lemmings_to_bevy;
mod xbrz;
mod main_menu;
mod level_selection_menu;
mod menu_common;
mod fadeout;
mod level_preview;
mod helpers;
mod ingame;
mod mouse_cursor;

use bevy::prelude::*;
use bevy::window::PresentMode;
use lemmings_to_bevy::load_lemmings_textures::GameTextures;
use lemmings::loader;

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
const ORIGINAL_GAME_W: usize = 320;
const ORIGINAL_GAME_H: usize = 200;
// I'm declaring an 'original game pixel' to be called a 'point'.
const POINT_SIZE: f32 = (RES_H as f32) / (ORIGINAL_GAME_H as f32); // How many bevy transform values to get one 'point' (pixel) in the original game.
const TEXTURE_SCALE: f32 = POINT_SIZE / (SCALE as f32);

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default, Copy)]
pub enum GameState {
    // #[default]
    MainMenu,
    LevelSelectionMenu,
    LevelPreview,
    #[default]
    InGame,
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
) {
    commands.spawn(Camera2dBundle::default());
}

fn main() {
    // TODO multithread this! https://doc.rust-lang.org/book/ch16-02-message-passing.html
    let games = loader::load().unwrap();
    let game = games.lemmings.unwrap();

    // TODO think about how all the assets are centered, so that they can be blurry maybe?
    // Especially seems to affect even numbered ones? Or odd?
    App::new()
        .add_state::<GameState>()
        .insert_resource(game)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        // TODO think about how to configure the window in bevy 0.10
        // .add_plugins(DefaultPlugins.set(WindowPlugin {
        //     window: WindowDescriptor {
        //         title: "Rusty Lemmings".to_string(),
        //         width: RES_W as f32,
        //         height: RES_H as f32,
        //         //resizable: false,
        //         present_mode: PresentMode::Fifo, // Battery-friendly vsync.
        //         ..Default::default()
        //     },
        //     ..default()
        // }))
        .add_plugin(lemmings_to_bevy::load_lemmings_textures::LoadLemmingsTexturesPlugin)
        .add_plugin(fadeout::FadeoutPlugin)
        .add_plugin(main_menu::MainMenuPlugin)
        .add_plugin(level_selection_menu::LevelSelectionMenuPlugin)
        .add_plugin(level_preview::LevelPreviewPlugin)
        .add_plugin(ingame::InGamePlugin)
        .add_plugin(mouse_cursor::MouseCursorPlugin)
        .add_startup_system(startup)
        .add_system(animate_sprite)
        .add_plugin(bevy_inspector_egui::quick::WorldInspectorPlugin::new())
        .run();
}
