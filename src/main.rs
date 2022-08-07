#![allow(dead_code)] // TODO disable once the app is mostly complete.

mod lemmings;
mod lemmings_to_bevy;
mod xbrz;

use bevy::{
    prelude::*,
    window::PresentMode,
};
use lemmings_to_bevy::load_lemmings_textures::GameTextures;

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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    MainMenu,
}

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
                translation: Vec3::new(0., 0., 2.),
                scale,
                ..default()
            },        
            ..default()
        });
        commands
        .spawn_bundle(SpriteBundle {
            texture: game_textures.mayhem.clone(),
            transform: Transform{
                translation: Vec3::new(0., -5. * POINT_SIZE, 3.),
                scale,
                ..default()
            },        
            ..default()
        });
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

fn post_startup_setup(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_textures.mining_right.clone(),
            transform: Transform{
                translation: Vec3::new(0., 0., 1.),
                scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
                ..default()
            },        
            ..default()
        })
        .insert(AnimationTimer(Timer::from_seconds(FRAME_DURATION, true)));
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_textures.blocking.clone(),
            transform: Transform{
                translation: Vec3::new(100., 0., 1.),
                scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
                ..default()
            },     
            ..default()
        })
        .insert(AnimationTimer(Timer::from_seconds(FRAME_DURATION, true)));
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

fn add_people(mut commands: Commands) {
    commands.spawn().insert(Person).insert(Name("Elaina Proctor".to_string()));
    commands.spawn().insert(Person).insert(Name("Renzo Hume".to_string()));
    commands.spawn().insert(Person).insert(Name("Zayna Nieves".to_string()));
}

struct GreetTimer(Timer);

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in query.iter() {
            println!("hello {}!", name.0);
        }
    }
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(GreetTimer(Timer::from_seconds(2.0, true)))
            .add_startup_system(add_people)
            .add_system(greet_people);
    }
}

fn main() {
    // TODO think about how all the assets are centered, so that they can be blurry maybe?
    // Especially seems to affect even numbered ones? Or odd?
    App::new()
        .add_state(GameState::MainMenu)
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .insert_resource(WindowDescriptor {
            title: "Rusty Lemmings".to_string(),
            width: RES_W as f32,
            height: RES_H as f32,
            //resizable: false,
            present_mode: PresentMode::Fifo, // Battery-friendly vsync.
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system_to_stage(StartupStage::PostStartup, post_startup_setup)
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_menu_background)
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_menu_logo)
        .add_plugin(HelloPlugin)
        .add_plugin(lemmings_to_bevy::load_lemmings_textures::LoadLemmingsTexturesPlugin)
        .add_system(animate_sprite)
        .add_system(animate_blinking_sprites)        
        .run();
}

/*
extern crate quicksilver;

mod lemmings;
mod qs_helpers;
mod scenes;
mod xbrz;

use std::boxed::Box;
use quicksilver::{
    Result,
    geom::{Rectangle, Vector, Transform},
    graphics::{Background::Col, Color},
    lifecycle::{Event, Settings, State, Window, run},
};
use qs_helpers::*;
use scenes::{Scene, EventAction, game_selection::GameSelection};
use crate::lemmings::{loader, levels_per_game_and_skill};
use scenes::level::LevelScene;

const FADE_FRAMES: isize = 20; // 40 is graceful like the original game.

struct GameController {
    scene: Box<dyn Scene>,
    is_fading_out: bool,
    is_fading_in: bool,
    fade: isize, // 0 = looks normal, FADE_FRAMES = looks black.
    can_update: bool, // Used to prevent it updating multiple times per draw, workaround for QS ignoring `max_updates: 1`.
}

impl State for GameController {
    fn new() -> Result<GameController> {

        // Jump direct to the game.
        let games = loader::load()?;
        let index: usize = 3;
        let game = games.lemmings.unwrap();
        let levels = levels_per_game_and_skill::levels_per_game_and_skill(&game.id, 0, &game.levels);
        let scene = Box::new(LevelScene::new(game, index, levels[index].clone())?);

        // let scene = Box::new(GameSelection::new()?);
        Ok(GameController { scene, is_fading_out: false, is_fading_in: false, fade: 0, can_update: true })
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        if !self.is_fading_in && !self.is_fading_out {
            let actions = self.scene.event(event, window)?;
            for action in actions {
                match action {
                    EventAction::BeginFadeOut => {
                        self.fade = 0;
                        self.is_fading_out = true;
                    }
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        // Disallow multiple updates per draw.
        if !self.can_update {
            return Ok(());
        }
        self.can_update = false;

        if self.is_fading_out {
            self.fade += 1;
            if self.fade >= FADE_FRAMES {
                self.fade = FADE_FRAMES;
                self.scene = self.scene.next_scene()?.unwrap();
                self.is_fading_out = false;
                self.is_fading_in = true;
            }
            Ok(())
        } else if self.is_fading_in {
            self.fade -= 1;
            if self.fade <= 0 {
                self.fade = 0;
                self.is_fading_in = false;
            }
            Ok(())
        } else {
            self.scene.update(window)
        }
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        self.can_update = true;

        self.scene.draw(window)?;

        if self.is_fading_in || self.is_fading_out {
            window.draw_ex(
                &Rectangle::new((0, 0), (SCREEN_WIDTH, SCREEN_HEIGHT)),
                Col(Color { r: 0., g: 0., b: 0., a: self.fade as f32 / FADE_FRAMES as f32 }),
                Transform::IDENTITY,
                999);            
        }

        Ok(())
    }
}

fn main() {
    run::<GameController>("Rusty Lemmings",
        Vector::new(SCREEN_WIDTH, SCREEN_HEIGHT),
        Settings {
            draw_rate: 1000. / 60.,
            max_updates: 1,
            update_rate: 1000. / 60.,
            ..Settings::default()
        });
}

// fn xmain() -> GameResult<()> {
//     let cb = ContextBuilder::new("lemmings", "anon")
//         .window_setup(conf::WindowSetup::default().title("Lemmings"))
//         .window_mode(conf::WindowMode::default().dimensions(1920, 1080));
//     let context = &mut cb.build()?;
//     let state = &mut MainState::new(context).unwrap();
//     event::run(context, state).unwrap();

//     // wait("Before loading"); // 412kb

//     // use std::time::Instant;
//     // let now = Instant::now();
//     // println!("Loading all");
//     // 
//     // let elapsed = now.elapsed();
//     // println!("Took: {:?}", elapsed); // 27ms optimised.

//     // wait("After loading"); // 18MB

//     // for game in games {
//     //     println!("Game: {}", game.name);
//     //     for (key, level) in game.levels {
//     //         println!("  Level: {:?}", level.name);
//     //     }
//     // }

//     // wait("After printing"); // 4.5MB


//     // for (key, level) in &games.lemmings.expect("Lemmings at least should load").levels {
//     //     // let key = 1;
//     //     // let level = &levels[&key];
//     //     println!("Level: {:?}", level.name);
//     //     // let rendered = render_level(level, &grounds, &specials)?;
//     //     // let buf = u32_to_u8_slice(&rendered.bitmap);
//     //     // let filename = format!("output/levels/{} {} ({} - {}).png", key, level.name, level.globals.normal_graphic_set, level.globals.extended_graphic_set);
//     //     // image::save_buffer(filename, &buf, rendered.size.width() as u32, LEVEL_HEIGHT as u32, image::RGBA(8)).unwrap();
//     // }

//     // let buf = u32_to_u8_slice(&image.bitmap);
//     // image::save_buffer("output/background.png", &buf, image.width as u32, image.height as u32, image::RGBA(8)).unwrap();

//     Ok(())
// }

*/