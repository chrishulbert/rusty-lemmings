use std::boxed::Box;

extern crate quicksilver;
use quicksilver::{
    Result,
    geom::{Rectangle, Vector, Transform},
    graphics::{Background::Col, Color},
    lifecycle::{Event, Settings, State, Window, run},
};

mod lemmings;

mod qs_helpers;
use qs_helpers::*;

mod scenes;
use scenes::{Scene, EventAction, game_selection::GameSelection};

mod xbrz;

/// Because QS doesn't let you control the init of your root State, this has to be an outer 'wrapper'
/// around our ref-counted Game.
struct GameController {
    scene: Box<dyn Scene>,
    is_fading_out: bool,
    is_fading_in: bool,
    fade: isize, // 0 = looks normal, FADE_FRAMES = looks black.
    can_update: bool, // Used to prevent it updating multiple times per draw, workaround for QS ignoring `max_updates: 1`.
}

const FADE_FRAMES: isize = 20; // 40 is graceful like the original game.

impl State for GameController {
    fn new() -> Result<GameController> {
        let scene = Box::new(GameSelection::new()?);
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

// #[derive(Debug, Copy, Clone)]
// struct LevelSize {
//     min_x: i32,
//     max_x: i32,
// }

// impl LevelSize {
//     fn width(&self) -> i32 {
//         self.max_x - self.min_x
//     }
// }

// const SPECIAL_LEFT_X: i32 = 320;

// fn size_of_level(level: &Level, grounds: &[GroundCombined]) -> LevelSize {
//     if level.globals.extended_graphic_set != 0 {
//         return LevelSize {
//             min_x: SPECIAL_LEFT_X,
//             max_x: SPECIAL_LEFT_X + special::WIDTH as i32,            
//         }
//     }

//     let mut size = LevelSize {
//         min_x: std::i32::MAX,
//         max_x: std::i32::MIN,
//     };
//     let ground = &grounds[level.globals.normal_graphic_set as usize];
//     for terrain in level.terrain.iter() {
//         let width = ground.ground.terrain_info[terrain.terrain_id as usize].width as i32;
//         size.min_x = cmp::min(size.min_x, terrain.x);
//         size.max_x = cmp::max(size.max_x, terrain.x + width);
//     }
//     size
// }

// struct RenderedLevel {
//     bitmap: Vec<u32>,
//     size: LevelSize,
// }

// const LEVEL_BACKGROUND: u32 = 0xff000000;
// const LEVEL_HEIGHT: i32 = 160;

// fn draw(sprite: &Vec<u32>,
//         x: i32, y: i32,
//         sprite_width: i32, sprite_height: i32,
//         canvas: &mut Vec<u32>, canvas_width: i32, canvas_height: i32,
//         do_not_overwrite_existing_terrain: bool,
//         is_upside_down: bool,
//         remove_terrain: bool,
//         must_have_terrain_underneath_to_be_visible: bool) {
//     let mut canvas_offset = y * canvas_width + x;
//     let canvas_stride = canvas_width - sprite_width;
//     let mut sprite_offset: i32 = if is_upside_down { (sprite_height - 1) * sprite_width } else { 0 };
//     let sprite_stride: i32 = if is_upside_down { -2 * sprite_width } else { 0 };
//     for pixel_y in 0..sprite_height {
//         if pixel_y+y < 0 || pixel_y+y >= canvas_height { // Out of bounds, skip a row.
//             canvas_offset += sprite_width + canvas_stride;
//             sprite_offset += sprite_width + sprite_stride;
//             continue
//         }

//         for pixel_x in 0..sprite_width {
//             if pixel_x+x < 0 || pixel_x+x >= canvas_width { // Out of canvas bounds, skip this pixel.
//                 sprite_offset += 1;
//                 canvas_offset += 1;
//                 continue
//             }

//             if remove_terrain {
//                 if sprite[sprite_offset as usize] != 0 {
//                     canvas[canvas_offset as usize] = LEVEL_BACKGROUND;
//                 }
//                 sprite_offset += 1;
//                 canvas_offset += 1;
//                 continue;
//             }
//             if do_not_overwrite_existing_terrain {
//                 if canvas[canvas_offset as usize] != LEVEL_BACKGROUND { // Skip the 'paint' if there's existing terrain.
//                     sprite_offset += 1;
//                     canvas_offset += 1;
//                     continue;
//                 }
//             }
//             if must_have_terrain_underneath_to_be_visible {
//                 if canvas[canvas_offset as usize] == LEVEL_BACKGROUND { // Skip the 'paint' if there's no existing terrain.
//                     sprite_offset += 1;
//                     canvas_offset += 1;
//                     continue;
//                 }
//             }
//             let pixel = sprite[sprite_offset as usize];
//             if pixel != 0 {
//                 canvas[canvas_offset as usize] = pixel;
//             }
//             sprite_offset += 1;
//             canvas_offset += 1;
//         }
//         canvas_offset += canvas_stride;
//         sprite_offset += sprite_stride;
//     }
// }

// fn render_level(level: &Level, grounds: &[GroundCombined], specials: &SpecialMap) -> io::Result<RenderedLevel> {
//     let size = size_of_level(level, grounds);
//     let width = size.width();
//     let height = LEVEL_HEIGHT;
//     let pixels = width * height;
//     let mut rendered_level = RenderedLevel {
//         bitmap: vec![LEVEL_BACKGROUND; pixels as usize],
//         size: size,
//     };
//     let ground = &grounds[level.globals.normal_graphic_set as usize];
//     if level.globals.extended_graphic_set == 0 {
//         for terrain in level.terrain.iter() {
//             let terrain_info = &ground.ground.terrain_info[terrain.terrain_id as usize];
//             let sprite = &ground.terrain_sprites[&terrain.terrain_id];
//             draw(&sprite,
//                 (terrain.x - size.min_x) as i32, terrain.y,
//                 terrain_info.width as i32, terrain_info.height as i32,
//                 &mut rendered_level.bitmap,
//                 width as i32, height as i32,
//                 terrain.do_not_overwrite_existing_terrain,
//                 terrain.is_upside_down,
//                 terrain.remove_terrain,
//                 false);
//         }
//     } else {
//         let special = &specials[&(level.globals.extended_graphic_set as i32 - 1)];
//         rendered_level.bitmap.copy_from_slice(&special.bitmap);
//     }
//     for object in level.objects.iter() {
//         let object_info = &ground.ground.object_info[object.obj_id as usize];
//         let sprite = &ground.object_sprites[&object.obj_id];
//         draw(&sprite,l.
//             (object.x - size.min_x) as i32, object.y as i32,
//             object_info.width as i32, object_info.height as i32,
//             &mut rendered_level.bitmap,
//             width as i32, height as i32,
//             object.modifier.is_do_not_overwrite_existing_terrain(),
//             object.is_upside_down,
//             false,
//             object.modifier.is_must_have_terrain_underneath_to_be_visible());
//     }
//     Ok(rendered_level)
// }

// fn ggez_image_from_lemmings_image(ctx: &mut Context, image: &Image) -> GameResult<graphics::Image> {
//     let mut rgba: Vec<u8> = Vec::with_capacity(image.bitmap.len() * 4);
//     for abgr in image.bitmap.iter() {
//         rgba.push(*abgr as u8);
//         rgba.push((*abgr >> 8) as u8);
//         rgba.push((*abgr >> 16) as u8);
//         rgba.push((*abgr >> 24) as u8);
//     }
//     graphics::Image::from_rgba8(ctx, image.width as u16, image.height as u16, &rgba)
// }

// struct MainState {
//     games: Games,
//     image: graphics::Image,
//     pos_x: f32,
//     rot: f32,
// }

// impl MainState {
//     fn new(_ctx: &mut Context) -> GameResult<MainState> {
//         let games = loader::load()?;
//         let image = ggez_image_from_lemmings_image(_ctx, &games.lemmings.as_ref().unwrap().main.main_menu.background)?;
//         let s = MainState { games: games, image: image, pos_x: 0.0, rot: 0.0 };
//         Ok(s)
//     }
// }

// impl event::EventHandler for MainState {
//     fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
//         self.pos_x = self.pos_x % 800.0 + 1.0;
//         self.rot += 0.0166;
//         Ok(())
//     }
//     fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
//         graphics::clear(ctx);

//         graphics::circle(ctx,
//             DrawMode::Fill,
//             Point2::new(self.pos_x, 380.0),
//             100.0,
//             2.0)?;
//         graphics::draw(ctx, &self.image, Point2::new(960.0, 540.0), self.rot)?;

//         graphics::present(ctx);
//         timer::yield_now();
//         Ok(())
//     }
// }

// // fn wait(reason: &str) {
// //     println!("{}, press any key...", reason);
// //     let mut a = String::new();
// //     let _discarded = io::stdin().read_line(&mut a);
// // }

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
