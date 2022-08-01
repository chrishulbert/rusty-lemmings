#![allow(dead_code)] // TODO disable once the app is mostly complete.

mod lemmings;
mod lemmings_to_bevy;
mod xbrz;

use std::{fs, path};
use bevy::{
    prelude::*,
    window::PresentMode,
};
use lemmings::models::Animation;

use crate::lemmings::{loader, png};

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
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

struct MyAtlas {
    data: Vec<u32>,
    cols: usize,
    rows: usize,
    width: usize,
    height: usize,
}
fn make_atlas_from_frames(frames: &Vec<Vec<u32>>, width: usize, height: usize) -> MyAtlas {
    // Figure out the size to go for for the atlas.
    // TODO special case if only one treat as not an anim? Handle that in the parser?
    let len = frames.len();
    let lenf = len as f32;
    let cols = lenf.sqrt().round() as usize;
    let divides_perfectly = len % cols == 0;
    let rows = if divides_perfectly { len / cols } else { len / cols + 1};
    let atlas_width = width * cols + (cols - 1); // 1px gap between each.
    let atlas_height = height * rows + (rows - 1);
    let mut atlas = Vec::<u32>::new();
    atlas.resize(atlas_width * atlas_height, 0);
    let mut col: usize = 0;
    let mut row: usize = 0;
    for frame in frames {
        let start_atlas_x = col * (width + 1);
        let mut atlas_y = row * (height + 1);
        for frame_y in 0..height {
            let mut atlas_x = start_atlas_x;
            for frame_x in 0..width {
                atlas[atlas_y * atlas_width + atlas_x] = frame[frame_y * width + frame_x];
                atlas_x += 1;
            }
            atlas_y += 1;
        }

        // Move to the next slot.
        col += 1;
        if col >= cols {
            col = 0;
            row += 1;
        }
    }
    MyAtlas{
        data: atlas,
        cols,
        rows,
        width: atlas_width,
        height: atlas_height,
    }
}

// 4k is 3840x2160
// 5K is 5120x2880
// Original game is 320x200
// Since it scrolls horizontally, i only care about height for scaling.
// 5k ratio is 14.4x high: could do 6x then 3x to get 18.
// 4k is 10.8x high
// Realistically: 6x then 2x to get 12: good enough for 4k.
// Or should we do 5x then 2x to get 10 and have a little margin for 4k?
fn scale_animation(a: &Animation) -> Animation {
    let scale_a: usize = 6;
    let scale_b: usize = 2;
    let mut big_frames = Vec::<Vec<u32>>::new();
    for frame in &a.frames {
        let bigger = xbrz::scale(scale_a as u8, frame, a.width as u32, a.height as u32);
        let biggest = xbrz::scale(scale_b as u8, &bigger, (a.width * scale_a) as u32, (a.height * scale_a) as u32);
        big_frames.push(biggest);
    }
    Animation{
        frames: big_frames,
        width: a.width * scale_a * scale_b,
        height: a.height * scale_a * scale_b,
    }
}

fn main() {
    let games = loader::load().unwrap();
    let game = games.lemmings.unwrap();
    let rusty_path = format!("{}/rusty", game.path);
    fs::create_dir_all(&rusty_path).unwrap();
    for asset in game.all_assets() {
        let filename_base = format!("{}/{}", rusty_path, asset.name);
        match asset.content {
            lemmings::models::AnimationOrImage::Animation(a) => {
                {
                    let atlas = make_atlas_from_frames(&a.frames, a.width, a.height);
                    let filename = format!("{}.original.{}r.{}c.{}w.{}h.png", filename_base, atlas.cols, atlas.rows, a.width, a.height);
                    if !path::Path::new(&filename).exists() {
                        let png = png::png_data(atlas.width as u32, atlas.height as u32, &atlas.data);
                        fs::write(filename, png).unwrap();
                    }
                }
                {
                    let scaled = scale_animation(&a);
                    let atlas = make_atlas_from_frames(&scaled.frames, scaled.width, scaled.height);
                    let filename = format!("{}.scaled.{}r.{}c.{}w.{}h.png", filename_base, atlas.cols, atlas.rows, scaled.width, scaled.height);
                    if !path::Path::new(&filename).exists() {
                        let png = png::png_data(atlas.width as u32, atlas.height as u32, &atlas.data);
                        fs::write(filename, png).unwrap();
                    }
                }

                // for (index, frame) in a.frames.iter().enumerate() {
                //     {
                //         let filename = format!("{}.original.{}.png", filename_base, index);
                //         if !path::Path::new(&filename).exists() {
                //             let png = png::png_data(a.width as u32, a.height as u32, &frame);
                //             fs::write(filename, png).unwrap();
                //         }
                //     }
                //     {
                //         let filename = format!("{}.scaled.{}.png", filename_base, index);
                //         if !path::Path::new(&filename).exists() {
                //             let bigger = xbrz::scale(6, &frame, a.width as u32, a.height as u32);
                //             let biggest = xbrz::scale(3, &bigger, (a.width * 6) as u32, (a.height * 6) as u32);
                //             let png = png::png_data((a.width * 6 * 3) as u32, (a.height * 6 * 3) as u32, &biggest);
                //             fs::write(filename, png).unwrap();
                //         }
                //     }
                // }
            },
            lemmings::models::AnimationOrImage::Image(i) => {
                {
                    let filename = format!("{}.original.png", filename_base);
                    if !path::Path::new(&filename).exists() {
                        let png = png::png_data(i.width as u32, i.height as u32, &i.bitmap);
                        fs::write(filename, png).unwrap();
                    }
                }
                {
                    let filename = format!("{}.scaled.png", filename_base);
                    if !path::Path::new(&filename).exists() {
                        let bigger = xbrz::scale(6, &i.bitmap, i.width as u32, i.height as u32);
                        let biggest = xbrz::scale(3, &bigger, (i.width * 6) as u32, (i.height * 6) as u32);
                        let png = png::png_data((i.width * 6 * 3) as u32, (i.height * 6 * 3) as u32, &biggest);
                        fs::write(filename, png).unwrap();
                    }
                }
            }
        }

    }
    // let bl = &game.main.lemming_animations.bashing_left;
    // for (index, frame) in bl.frames.iter().enumerate() {
    //     let filename = format!("{}/lemming_bashing_left_original_{}.png", rusty_path, index);
    //     if !path::Path::new(&filename).exists() {
    //     }

    //     let filename = format!("{}/lemming_bashing_left_scaled_{}.png", rusty_path, index);
    //     if !path::Path::new(&filename).exists() {
    //         let bigger = xbrz::scale(6, frame, bl.width as u32, bl.height as u32);
    //         let biggest = xbrz::scale(3, &bigger, (bl.width * 6) as u32, (bl.height * 6) as u32);
    //         let png = png::png_data((bl.width * 6 * 3) as u32, (bl.height * 6 * 3) as u32, &biggest);
    //         fs::write(filename, png).unwrap();
    //     }
    // }

    App::new()
        .insert_resource(WindowDescriptor {
            title: "Rusty Lemmings".to_string(),
            width: 1280.,
            height: 720.,
            present_mode: PresentMode::Fifo, // Battery-friendly vsync.
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_plugin(HelloPlugin)
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