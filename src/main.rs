#![allow(dead_code)] // TODO disable once the app is mostly complete.

mod lemmings;
mod lemmings_to_bevy;
mod xbrz;

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
const TEXTURE_SCALE: f32 = (RES_H as f32) / (ORIGINAL_GAME_H as f32) / (SCALE as f32);

// use std::{fs, path};
use bevy::{
    prelude::*,
    window::PresentMode,
};
use lemmings_to_bevy::load_lemmings_textures::GameTextures;

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

fn post_startup_setup(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_textures.mining_right.clone(),
            transform: Transform{
                translation: Vec3::new(-100., 0., 0.),
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
                translation: Vec3::new(100., 0., 0.),
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
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Rusty Lemmings".to_string(),
            width: RES_W as f32,
            height: RES_H as f32,
            resizable: false,
            present_mode: PresentMode::Fifo, // Battery-friendly vsync.
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system_to_stage(StartupStage::PostStartup, post_startup_setup)
        .add_plugin(HelloPlugin)
        .add_plugin(lemmings_to_bevy::load_lemmings_textures::LoadLemmingsTexturesPlugin)
        .add_system(animate_sprite)
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