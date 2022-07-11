// This is the scene for playing an actual level.
// Apologies for anyone reading this code, was written while learning Rust. The plan is to refactor to be 
// idiomatic and neater once I actually figure out the game logic - Chris.
// Lemmings fun 4 - now use miners and climbers is a good test for clipping.

extern crate image;
use std::mem;
use std::slice;

extern crate quicksilver;
use quicksilver::{
    Result,
    geom::{Rectangle, Transform, Vector},
    graphics::{Background::Img, Background::Col, Color, Image as QSImage},
    lifecycle::{Event, Window},
    input::MouseButton,
};

use std::collections::HashMap;
use crate::lemmings::models::*;
use Scene;
use EventAction;
use qs_helpers::*;
use crate::lemmings::level_renderer::{self, RenderedLevel};
use xbrz;

const SKILL_PANEL_SCALE: u8 = 5;
const SKILL_WIDTH: f32 = 16.;
const SKILL_HEIGHT: f32 = 24.;
const SKILL_PANEL_GRAPHIC_HEIGHT: f32 = 40.;
const SKILL_BUTTONS: usize = 12;
const AUTO_SCROLL_SLOW_MARGIN: f32 = 30.;
const AUTO_SCROLL_FAST_MARGIN: f32 = 10.;
const GAME_AREA_HEIGHT: f32 = 960.;
const OBJECT_ID_EXIT: usize = 0;
const OBJECT_ID_START: usize = 1;

type SliceMap = HashMap<isize, QSImage>;
type ObjectMap = HashMap<usize, Vec<QSImage>>;
type ObjectFrames = HashMap<usize, usize>; // For remembering which animation frame each object is up to.

// const STRIP_WIDTH_GAME_PX: usize = 11; // This way each vertical strip of the map is just < 1MB. Eg 160 high * 12 scale * 4 rgba * 11 game pixels wide * 12 scale = 0.97MB

#[derive(Debug, Clone)]
pub enum LemmingAction {
    Normal,
    // Skills:
    Climbing,
    Parachuting,
    Exploding,
    Blocking,
    Building,
    Bashing,
    DiggingDiagonally, // AKA mining.
    DiggingDown,

    // Other:
    Jumping, // Walking up a 3-6px step.
    Falling,
    Drowning,
    Exiting,
    // TODO others as per models>LemmingAnimations
}

impl LemmingAction {
    fn action_for_selected_skill_index(index: isize) -> LemmingAction {
        match index {
            0 => LemmingAction::Climbing,
            1 => LemmingAction::Parachuting,
            2 => LemmingAction::Exploding,
            3 => LemmingAction::Blocking,
            4 => LemmingAction::Building,
            5 => LemmingAction::Bashing,
            6 => LemmingAction::DiggingDiagonally,
            7 => LemmingAction::DiggingDown,
            _ => LemmingAction::Normal,
        }  
    }
}

// TODO move this into a separate file.
pub struct ScaledLemmingAnimations {
    pub walking_right: Vec<QSImage>,
    pub jumping_right: Vec<QSImage>, // Walking up a step 3-6px tall.
    pub walking_left: Vec<QSImage>,
    pub jumping_left: Vec<QSImage>,
    pub digging: Vec<QSImage>,
    pub climbing_right: Vec<QSImage>,
    pub climbing_left: Vec<QSImage>,
    pub drowning: Vec<QSImage>,
    pub post_climb_right: Vec<QSImage>,
    pub post_climb_left: Vec<QSImage>,
    pub brick_laying_right: Vec<QSImage>,
    pub brick_laying_left: Vec<QSImage>,
    pub bashing_right: Vec<QSImage>,
    pub bashing_left: Vec<QSImage>,
    pub mining_right: Vec<QSImage>,
    pub mining_left: Vec<QSImage>,
    pub falling_right: Vec<QSImage>,
    pub falling_left: Vec<QSImage>,
    pub pre_umbrella_right: Vec<QSImage>,
    pub umbrella_right: Vec<QSImage>,
    pub pre_umbrella_left: Vec<QSImage>,
    pub umbrella_left: Vec<QSImage>,
    pub splatting: Vec<QSImage>,
    pub exiting: Vec<QSImage>,
    pub fried: Vec<QSImage>,
    pub blocking: Vec<QSImage>,
    pub shrugging_right: Vec<QSImage>, // Builder running out of bricks.
    pub shrugging_left: Vec<QSImage>,
    pub oh_no_ing: Vec<QSImage>,
    pub explosion: Vec<QSImage>,
}

fn scale_animation(animation: &Animation) -> Result<Vec<QSImage>> {
    let mut qs_frames: Vec<QSImage> = Vec::new();
    for frame in animation.frames.iter() {
        let image = Image {
            bitmap: frame.to_vec(),
            width: animation.width,
            height: animation.height,
        };
        let qs_frame = qs_image_from_lemmings_image_scaled_twice(&image, SCALE, 2)?;
        qs_frames.push(qs_frame);
    }    
    Ok(qs_frames)
}

impl ScaledLemmingAnimations {
    fn new(lemmingAnimations: &LemmingAnimations) -> Result<ScaledLemmingAnimations> {
        Ok(ScaledLemmingAnimations {
            walking_right: scale_animation(&lemmingAnimations.walking_right)?,
            jumping_right: scale_animation(&lemmingAnimations.jumping_right)?,
            walking_left: scale_animation(&lemmingAnimations.walking_left)?,
            jumping_left: scale_animation(&lemmingAnimations.jumping_left)?,
            digging: scale_animation(&lemmingAnimations.digging)?,
            climbing_right: scale_animation(&lemmingAnimations.climbing_right)?,
            climbing_left: scale_animation(&lemmingAnimations.climbing_left)?,
            drowning: scale_animation(&lemmingAnimations.drowning)?,
            post_climb_right: scale_animation(&lemmingAnimations.post_climb_right)?,
            post_climb_left: scale_animation(&lemmingAnimations.post_climb_left)?,
            brick_laying_right: scale_animation(&lemmingAnimations.brick_laying_right)?,
            brick_laying_left: scale_animation(&lemmingAnimations.brick_laying_left)?,
            bashing_right: scale_animation(&lemmingAnimations.bashing_right)?,
            bashing_left: scale_animation(&lemmingAnimations.bashing_left)?,
            mining_right: scale_animation(&lemmingAnimations.mining_right)?,
            mining_left: scale_animation(&lemmingAnimations.mining_left)?,
            falling_right: scale_animation(&lemmingAnimations.falling_right)?,
            falling_left: scale_animation(&lemmingAnimations.falling_left)?,
            pre_umbrella_right: scale_animation(&lemmingAnimations.pre_umbrella_right)?,
            umbrella_right: scale_animation(&lemmingAnimations.umbrella_right)?,
            pre_umbrella_left: scale_animation(&lemmingAnimations.pre_umbrella_left)?,
            umbrella_left: scale_animation(&lemmingAnimations.umbrella_left)?,
            splatting: scale_animation(&lemmingAnimations.splatting)?,
            exiting: scale_animation(&lemmingAnimations.exiting)?,
            fried: scale_animation(&lemmingAnimations.fried)?,
            blocking: scale_animation(&lemmingAnimations.blocking)?,
            shrugging_right: scale_animation(&lemmingAnimations.shrugging_right)?,
            shrugging_left: scale_animation(&lemmingAnimations.shrugging_left)?,
            oh_no_ing: scale_animation(&lemmingAnimations.oh_no_ing)?,
            explosion: scale_animation(&lemmingAnimations.explosion)?,
        })
    }
    fn frames_for_action(&self, action: LemmingAction, is_right: bool) -> &Vec<QSImage> {
        match action {
            LemmingAction::Normal => { if is_right { &self.walking_right } else { &self.walking_left } },
            LemmingAction::Falling => { if is_right { &self.falling_right } else { &self.falling_left } },
            LemmingAction::Bashing => { if is_right { &self.bashing_right } else { &self.bashing_left } },
            LemmingAction::DiggingDiagonally => { if is_right { &self.mining_right } else { &self.mining_left } },
            LemmingAction::DiggingDown => { &self.digging },
            _ => { if is_right { &self.walking_right } else { &self.walking_right } },
        }
    }
}

pub struct InGameLemming {
    x: isize, // In game pixels, center.
    y: isize,
    is_facing_right: bool, // false = facing left.
    animation_frame: usize, // No upper bound, you're supposed to % this by the number of frames when drawing. Reset to zero when starting eg a new bash action though.
    action: LemmingAction,
}

// The finite-state-machine of the game, eg you start with a brief countdown, then open the door, then play.
pub enum InGameState {
    CountdownToOpenDoor(isize),
    OpeningDoor(usize), // frame can be larger than the number of frames for the door opening, this gives a bit extra delay before lemmings pop out.
    Playing,
}

pub struct LevelScene {
    game: Game,
    level_index: usize,
    level: Level,
    font: QSGameFont,
    skill_panel: QSImage,
    mouse_was_down: bool,
    level_unscaled: RenderedLevel, // Source of truth for where the lemmings can walk
    level_scaled_image: Image,
    level_slices: SliceMap,
    scroll_offset_x: isize, // In game pixels.
    selected_skill_index: isize,
    objects_prescaled: ObjectMap,
    object_animation_frames: ObjectFrames,
    frame_counter: usize, // 60fps.
    game_frame_counter: usize, // 15fps.
    lemmings: Vec<InGameLemming>,
    state: InGameState,
    lemming_out_due: usize, // game frame a lemming is due to fall out.
    lemmings_dropped: usize,
    scaled_lemming_animations: ScaledLemmingAnimations,
}

fn u32_to_u8_slice(original: &[u32]) -> &[u8] {
    let count = original.len() * mem::size_of::<u32>();
    let ptr = original.as_ptr() as *const u8;
    return unsafe { slice::from_raw_parts(ptr, count) };
}

fn export_anim(a: &Animation, f: &str) {
    for (index, frame) in a.frames.iter().enumerate() {
        let filename = format!("{}_{}.png", f, index);
        let u8s = u32_to_u8_slice(&frame);
        image::save_buffer(filename, u8s, a.width as u32, a.height as u32, image::RGBA(8)).unwrap();
    }
}

fn export(l: &LemmingAnimations) {
    export_anim(&l.walking_right, "output/walking_right");
    export_anim(&l.walking_right, "output/walking_right");
    export_anim(&l.jumping_right, "output/jumping_right");
    export_anim(&l.walking_left, "output/walking_left");
    export_anim(&l.jumping_left, "output/jumping_left");
    export_anim(&l.digging, "output/digging");
    export_anim(&l.climbing_right, "output/climbing_right");
    export_anim(&l.climbing_left, "output/climbing_left");
    export_anim(&l.drowning, "output/drowning");
    export_anim(&l.post_climb_right, "output/post_climb_right");
    export_anim(&l.post_climb_left, "output/post_climb_left");
    export_anim(&l.brick_laying_right, "output/brick_laying_right");
    export_anim(&l.brick_laying_left, "output/brick_laying_left");
    export_anim(&l.bashing_right, "output/bashing_right");
    export_anim(&l.bashing_left, "output/bashing_left");
    export_anim(&l.mining_right, "output/mining_right");
    export_anim(&l.mining_left, "output/mining_left");
    export_anim(&l.falling_right, "output/falling_right");
    export_anim(&l.falling_left, "output/falling_left");
    export_anim(&l.pre_umbrella_right, "output/pre_umbrella_right");
    export_anim(&l.umbrella_right, "output/umbrella_right");
    export_anim(&l.pre_umbrella_left, "output/pre_umbrella_left");
    export_anim(&l.umbrella_left, "output/umbrella_left");
    export_anim(&l.splatting, "output/splatting");
    export_anim(&l.exiting, "output/exiting");
    export_anim(&l.fried, "output/fried");
    export_anim(&l.blocking, "output/blocking");
    export_anim(&l.shrugging_right, "output/shrugging_right");
    export_anim(&l.shrugging_left, "output/shrugging_left");
    export_anim(&l.oh_no_ing, "output/oh_no_ing");
    export_anim(&l.explosion, "output/explosion");
}

fn mask_to_u32_slice(original: &[u8]) -> Vec<u32> {
    let mut v: Vec<u32> = Vec::new();
    for o in original.iter() {
        if *o == 0 {
            v.push(0);
        } else {
            v.push(0xff000000);
        }
    }
    v
}

fn export_mask(m: &Mask, f: &str) {
    for (index, frame) in m.frames.iter().enumerate() {
        let filename = format!("{}_{}.png", f, index);
        let u32s = mask_to_u32_slice(&frame);
        let u8s = u32_to_u8_slice(&u32s);
        image::save_buffer(filename, u8s, m.width as u32, m.height as u32, image::RGBA(8)).unwrap();
    }
}

fn export_masks(m: &Masks) {
    export_mask(&m.bash_right, "output/bash_right");
    export_mask(&m.bash_left, "output/bash_left");
    export_mask(&m.mine_right, "output/mine_right");
    export_mask(&m.mine_left, "output/mine_left");
    export_mask(&m.explosion, "output/explosion");
}


// Skill panel only gets buttons displayed. At 5x, thus covers 5*24=120pt. Level is 160x6=960. 120+960 = 1080 full.
// Only regret is that the buttons don't fill the whole width, but it looks good left-aligned with text to its right.
// Skill panel minimap has 2-game-px margin on top and bottom, thus 20-game-pt high (1/8th original game px). Aka 20*5=100pt / 200px high.
// Map is 1040px wide, aka 520pt wide, aka 104-game-px wide. Thus supporting a max map width of 832px.
impl LevelScene {
    pub fn new(game: Game, level_index: usize, level: Level) -> Result<LevelScene> {
        // export(&game.main.lemming_animations);
        export_masks(&game.main.masks);        

        let font = game.main.game_font.qs_font()?;
        let skill_panel = qs_image_from_lemmings_image_scaled_twice(&game.main.skill_panel, SKILL_PANEL_SCALE, 2)?;

        // Pre-render the objects used in this level.
        let mut objects_prescaled = ObjectMap::new();
        let mut object_animation_frames = ObjectFrames::new();
        {
            let ground = &game.grounds[&(level.globals.normal_graphic_set as i32)];
            for object in level.objects.iter() {
                let animation = &ground.object_sprites[&(object.obj_id as i32)];
                let qs_frames = scale_animation(animation)?;
                objects_prescaled.insert(object.obj_id, qs_frames);
                object_animation_frames.insert(object.obj_id, 0);
            }
        }

        // Pre-render other graphics.
        let scaled_lemming_animations = ScaledLemmingAnimations::new(&game.main.lemming_animations)?;

        println!("Rendering level");
        let render = level_renderer::render(&level, &game.grounds, &game.specials, false)?; // This renders in game pixels.
        println!("Scaling level A");
        let scaled_half = xbrz::scale(SCALE, &render.image.bitmap, render.image.width as u32, render.image.height as u32); // Scale the whole thing in one go so there are no seams.
        println!("Scaling level B");
        let scaled = xbrz::scale(2, &scaled_half, render.image.width as u32 * SCALE as u32, render.image.height as u32 * SCALE as u32);
        let scaled_width: usize = render.image.width * SCALE as usize * 2;
        let scaled_height: usize = render.image.height * SCALE as usize * 2;
        println!("Slicing level");
        // Slice it into game pixels.
        let mut slices = SliceMap::new();
        for image_x in 0..render.image.width {
            let slice_qs_image = slice(&scaled, scaled_width, scaled_height, image_x)?;
            let game_x: isize = image_x as isize + render.size.min_x;
            slices.insert(game_x, slice_qs_image);
        }
        println!("Finished slicing");

        let level_scaled_image = Image {
            bitmap: scaled,
            width: scaled_width,
            height: scaled_height,
        };
        let scroll_offset_x = level.globals.start_screen_xpos as isize;
        Ok(LevelScene {
            game,
            level_index,
            level,
            font,
            skill_panel,
            mouse_was_down: false,
            level_unscaled: render,
            level_scaled_image,
            level_slices: slices,
            scroll_offset_x,
            selected_skill_index: 0,
            objects_prescaled,
            object_animation_frames,
            frame_counter: 0,
            game_frame_counter: 0,
            lemmings: Vec::new(),
            state: InGameState::CountdownToOpenDoor(5),
            lemming_out_due: usize::max_value(),
            lemmings_dropped: 0,
            scaled_lemming_animations,
        })
    }

    fn on_mouse_down_in_game_area(&mut self, mouse: &Vector) {
        // Did they click on a lemming?
        if let Some(lemming_index) = self.closest_lemming_to_mouse(mouse) {
            let new_action = LemmingAction::action_for_selected_skill_index(self.selected_skill_index);
            self.lemmings[lemming_index].animation_frame = match new_action {
                LemmingAction::DiggingDiagonally => 1, // Digging starts on 1, i guess because 0 has a y movement.
                _ => 0,
            };
            self.lemmings[lemming_index].action = new_action;
        }
    }

    fn on_mouse_down_in_skill_bar(&mut self, mouse: &Vector) {
        let index = mouse.x as isize / (SKILL_WIDTH as isize * SKILL_PANEL_SCALE as isize);
        if index < SKILL_BUTTONS as isize {
            match index {
                // 0 => { - },ˀˀ
                // 1 => { + },
                2 => { self.selected_skill_index = index - 2 }, // climb
                3 => { self.selected_skill_index = index - 2 }, // umbrella
                4 => { self.selected_skill_index = index - 2 }, // explode
                5 => { self.selected_skill_index = index - 2 }, // block
                6 => { self.selected_skill_index = index - 2 }, // build
                7 => { self.selected_skill_index = index - 2 }, // bash
                8 => { self.selected_skill_index = index - 2 }, // diagonal dig
                9 => { self.selected_skill_index = index - 2 }, // vertical dig
                // 10 >= {  pause },
                // 11 >= {  explode all },
                _ => {},
            }
        }
    }

    fn screen_x_from_game_x(&self, game_x: isize) -> f32 {
        return (game_x - self.scroll_offset_x) as f32 * SCALE as f32;
    }

    fn screen_y_from_game_y(&self, game_y: isize) -> f32 {
        return game_y as f32 * SCALE as f32;
    }

    // Returns the index of the lemming under the mouse.
    fn closest_lemming_to_mouse(&self, mouse_pos: &Vector) -> Option<usize> {
        let mouse_game_x = mouse_pos.x as isize / SCALE as isize + self.scroll_offset_x;
        let mouse_game_y = mouse_pos.y as isize  / SCALE as isize;
        let mut closest_index: Option<usize> = Option::None;
        let mut closest_distance: isize = isize::max_value();
        for (i, lemming) in self.lemmings.iter().enumerate() {
            let this_distance: isize = (lemming.x - mouse_game_x).abs() + (lemming.y - mouse_game_y).abs();
            if this_distance < 20 && this_distance < closest_distance {
                closest_distance = this_distance;
                closest_index = Option::Some(i);
            }
        }
        closest_index
    }

    // To be called from update. This moves the lemmings.
    fn update_lemmings(&mut self) {
        for lemming in self.lemmings.iter_mut() {
            let is_clear_under = self.level_unscaled.is_clear_under_lemming(lemming.x, lemming.y);
            match lemming.action {
                LemmingAction::Normal => {
                    if is_clear_under {
                        lemming.y += 1;
                        lemming.action = LemmingAction::Falling;
                    } else { // On solid ground.
                        // On solid ground.
                        if lemming.is_facing_right {
                            if self.level_unscaled.is_clear_to_right_of_lemming(lemming.x, lemming.y) {
                                lemming.x += 1;
                                // TODO now check if directly under and to the direction faced is clear, if so drop 1px without falling.
                            } else {
                                lemming.is_facing_right = !lemming.is_facing_right; // Turn around.
                            }
                        } else {
                            if self.level_unscaled.is_clear_to_left_of_lemming(lemming.x, lemming.y) {
                                lemming.x -= 1;
                                // TODO now check if directly under and to the direction faced is clear, if so drop 1px without falling.
                            } else {
                                lemming.is_facing_right = !lemming.is_facing_right; // Turn around.
                            }
                        }
                    }
                    lemming.animation_frame += 1;
                },
                LemmingAction::Falling => {
                    if is_clear_under {
                        lemming.y += 1;
                    } else {
                        lemming.action = LemmingAction::Normal;
                    }
                    lemming.animation_frame += 1;
                },
                LemmingAction::DiggingDown => {
                    if is_clear_under {
                        lemming.action = LemmingAction::Falling;
                    } else {
                        lemming.animation_frame += 1;
                        let frames = self.scaled_lemming_animations.digging.len();
                        let frame_index = lemming.animation_frame % frames;
                        if frame_index == 4 || frame_index == 12 { // Just as he pulls his arm up.
                            lemming.y += 1;
                            mask_dig(&mut self.level_unscaled, 
                                    &mut self.level_scaled_image,
                                    &mut self.level_slices,
                                    lemming.x - 4, lemming.y + 4,
                                    9); // Clear 9 game px wide
                        }
                    }                    
                },
                LemmingAction::DiggingDiagonally => {
                    lemming.animation_frame += 1;
                    let frames = if lemming.is_facing_right { self.scaled_lemming_animations.mining_right.len() } else { self.scaled_lemming_animations.mining_left.len() };
                    let x_offset: isize = if lemming.is_facing_right { 1 } else { -1 };
                    let mask = if lemming.is_facing_right { &self.game.main.masks.mine_right } else { &self.game.main.masks.mine_left };
                    let frame_index = lemming.animation_frame % frames;
                    // Only some frames need you to actually move the lemming.
                    if frame_index == 0 {
                        lemming.y += 1;
                    }
                    if frame_index == 2 {
                        lemming.x += x_offset * 2;
                        lemming.y += 1;
                    }
                    if frame_index == 15 {
                        lemming.x += x_offset * 2;
                    }
                    // Only some frames mask.
                    if frame_index == 1 { // The top mask is manually offset so there's no gaps with the next mask.
                        let mask_frame = &mask.frames[0];
                        mask_area(&mut self.level_unscaled, 
                            &mut self.level_scaled_image,
                            &mut self.level_slices,
                            lemming.x - mask.width/2 + 1, lemming.y - mask.height/2 - 1,
                            mask_frame, mask.width, mask.height);
                    }
                    if frame_index == 2 {
                        let mask_frame = &mask.frames[1];
                        mask_area(&mut self.level_unscaled, 
                            &mut self.level_scaled_image,
                            &mut self.level_slices,
                            lemming.x - mask.width/2, lemming.y - mask.height/2 - 1,
                            mask_frame, mask.width, mask.height);
                    }
                },
                LemmingAction::Bashing => {
                    if is_clear_under {
                        lemming.action = LemmingAction::Falling;
                    } else {
                        if lemming.is_facing_right {
                            // if self.level_unscaled.is_clear_to_right_of_lemming(lemming.x, lemming.y) {
                            //     lemming.action = LemmingAction::Normal;
                            // } else {
                                lemming.animation_frame += 1;

                                // Only some frames need you to actually move the lemming.
                                let frames = self.scaled_lemming_animations.bashing_right.len();
                                let frame_index = lemming.animation_frame % frames;
                                if frame_index == 11 || frame_index == 12 || frame_index == 13 || frame_index == 14 ||
                                    frame_index == 27 || frame_index == 28 || frame_index == 29 || frame_index == 30 {
                                    lemming.x += 1;
                                }

                                // Only some frames actually 'bash' any of the ground out.
                                let mask = &self.game.main.masks.bash_right;
                                let mask_frame_index: isize = match frame_index {
                                    2 => 0,
                                    3 => 1,
                                    4 => 2,
                                    5 => 3,
                                    18 => 0,
                                    19 => 1,
                                    20 => 2,
                                    21 => 3,
                                    _ => -1,
                                };
                                if mask_frame_index >= 0 {
                                    let mask_frame = &mask.frames[mask_frame_index as usize];
                                    mask_area(&mut self.level_unscaled, 
                                        &mut self.level_scaled_image,
                                        &mut self.level_slices,
                                        lemming.x - mask.width/2, lemming.y - mask.height/2,
                                        mask_frame, mask.width, mask.height);
                                }
                            // }
                        } else {
                            if self.level_unscaled.is_clear_to_left_of_lemming(lemming.x, lemming.y) {
                                lemming.action = LemmingAction::Normal;
                            } else {
                                lemming.animation_frame += 1;

                                // Only some frames need you to actually move the lemming.
                                let frames = self.scaled_lemming_animations.bashing_left.len();
                                let frame_index = lemming.animation_frame % frames;
                                if frame_index == 11 || frame_index == 12 || frame_index == 13 || frame_index == 14 ||
                                    frame_index == 27 || frame_index == 28 || frame_index == 29 || frame_index == 30 {
                                    lemming.x -= 1;
                                }

                                // Only some frames actually 'bash' any of the ground out.
                                let mask = &self.game.main.masks.bash_left;
                                let mask_frame_index: isize = match frame_index {
                                    2 => 0,
                                    3 => 1,
                                    4 => 2,
                                    5 => 3,
                                    18 => 0,
                                    19 => 1,
                                    20 => 2,
                                    21 => 3,
                                    _ => -1,
                                };
                                if mask_frame_index >= 0 {
                                    let mask_frame = &mask.frames[mask_frame_index as usize];
                                    mask_area(&mut self.level_unscaled, 
                                        &mut self.level_scaled_image,
                                        &mut self.level_slices,
                                        lemming.x - mask.width/2, lemming.y - mask.height/2,
                                        mask_frame, mask.width, mask.height);
                                }
                            }
                        }
                    }
                },
                _ => {}, // TODO
            }
        }
    }

    // To be called from update.
    fn drop_lemming_if_necessary(&mut self) {
        let ground = &self.game.grounds[&(self.level.globals.normal_graphic_set as i32)];
        if self.game_frame_counter >= self.lemming_out_due { // Drop a lemming.                        
            // Count the starts, so if there are 2+, we can stagger where the lemmings fall.
            let mut starts_count: usize = 0;
            for object in self.level.objects.iter() {
                if object.obj_id == OBJECT_ID_START {
                    starts_count += 1;
                }
            }
            // Choose the correct start door.
            let mut x: isize = 0;
            let mut y: isize = 0;
            {
                let start_index_to_use = self.lemmings_dropped % starts_count;
                let mut start_index_found: usize = 0;
                for object in self.level.objects.iter() {
                    if object.obj_id == OBJECT_ID_START {
                        if start_index_found == start_index_to_use {
                            let animation = &ground.object_sprites[&(OBJECT_ID_START as i32)];
                            x = object.x as isize + animation.width as isize / 2;
                            y = object.y as isize + animation.height as isize / 2;
                            break;
                        }
                        start_index_found += 1;
                    }
                }
            }
            let lemming = InGameLemming {
                x, y,
                is_facing_right: true, // TODO which way?
                animation_frame: 0,
                action: LemmingAction::Falling,
            };
            self.lemmings.push(lemming);
            self.lemming_out_due = self.game_frame_counter + 20; // TODO make this proportional to the trapdoor speed.
            self.lemming_out_due = self.game_frame_counter + 9999999; // TODO make this proportional to the trapdoor speed.
            self.lemmings_dropped += 1;
        }
    }

}




impl Scene for LevelScene {
    fn event(&mut self, event: &Event, window: &mut Window) -> Result<Vec<EventAction>> {
        let mouse_pos = window.mouse().pos();
        let actions: Vec<EventAction> = Vec::new();
        match event {
            Event::MouseButton(MouseButton::Left, state) => {
                if !self.mouse_was_down && state.is_down() { // Start a click.
                    if mouse_pos.y > GAME_AREA_HEIGHT {
                        self.on_mouse_down_in_skill_bar(&mouse_pos)
                    } else {
                        self.on_mouse_down_in_game_area(&mouse_pos)
                    }
                }
                self.mouse_was_down = state.is_down();
            },
            _ => {}
        };
        Ok(actions)
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {        
        let mouse_pos = window.mouse().pos();

        if mouse_pos.y < GAME_AREA_HEIGHT {
            if mouse_pos.x < AUTO_SCROLL_FAST_MARGIN * SCALE as f32 {
                self.scroll_offset_x -= 2;
            } else if mouse_pos.x > SCREEN_WIDTH - AUTO_SCROLL_FAST_MARGIN * SCALE as f32 {
                self.scroll_offset_x += 2;
            } else if mouse_pos.x < AUTO_SCROLL_SLOW_MARGIN * SCALE as f32 {
                self.scroll_offset_x -= 1;
            } else if mouse_pos.x > SCREEN_WIDTH - AUTO_SCROLL_SLOW_MARGIN * SCALE as f32 {
                self.scroll_offset_x += 1;
            }

            // Clamp it to half a screen beyond the edges of the map.
            let half: isize = SCREEN_WIDTH as isize / SCALE as isize / 2;
            let min_x = self.level_unscaled.size.min_x - half;
            let max_x = self.level_unscaled.size.max_x - half;
            if self.scroll_offset_x < min_x {
                self.scroll_offset_x = min_x;
            } else if self.scroll_offset_x > max_x {
                self.scroll_offset_x = max_x;
            }
        }

        // Animate the object frames.
        let is_game_frame = (self.frame_counter % 4) == 0; // We only update at 15fps.
        if is_game_frame {
            self.game_frame_counter += 1;

            {
                let ground = &self.game.grounds[&(self.level.globals.normal_graphic_set as i32)];
                for object in self.level.objects.iter() {
                    let animation = &ground.object_sprites[&(object.obj_id as i32)];
                    let current_frame = self.object_animation_frames[&object.obj_id];
                    let new_frame = (current_frame + 1) % animation.frames.len();
                    self.object_animation_frames.insert(object.obj_id, new_frame);
                }
            }

            // Progress the state machine.
            match self.state {
                InGameState::CountdownToOpenDoor(countdown) => {
                    if countdown > 1 {
                        self.state = InGameState::CountdownToOpenDoor(countdown - 1);
                    } else {
                        self.state = InGameState::OpeningDoor(1);
                    }                    
                },
                InGameState::OpeningDoor(frame) => {
                    let ground = &self.game.grounds[&(self.level.globals.normal_graphic_set as i32)];
                    let door_animation = &ground.object_sprites[&(OBJECT_ID_START as i32)];
                    if frame < door_animation.frames.len() + 5 {
                        self.state = InGameState::OpeningDoor(frame + 1);
                    } else {
                        self.state = InGameState::Playing;
                        self.lemming_out_due = self.game_frame_counter;
                    }
                },
                InGameState::Playing => {
                    self.update_lemmings();
                    self.drop_lemming_if_necessary();
                },
            }

        }

        self.frame_counter += 1;

        std::thread::yield_now();
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        let mouse = window.mouse();
        let mouse_pos = mouse.pos();

        window.clear(Color::BLACK)?;

        // Bottom panel.
        {
            // Skills.
            let size = self.skill_panel.area().size;
            let w: f32 = size.x / 2.;
            let h: f32 = size.y / 2.;
            let x: f32 = 0.;
            let y: f32 = (SCREEN_HEIGHT - h).round();
            window.draw_ex(&Rectangle::new((x, y), (w, h)), Img(&self.skill_panel), Transform::IDENTITY, 1);

            // Text.
            let bar_height: f32 = (h * SKILL_HEIGHT / SKILL_PANEL_GRAPHIC_HEIGHT).round();
            let text_y_mid: f32 = SCREEN_HEIGHT - (bar_height/2.).round();
            let text_y: f32 = text_y_mid - 9. * SCALE as f32;
            let text_y_2: f32 = text_y_mid + 1. * SCALE as f32;
            let text_x: f32 = w + 2. * SCALE as f32;
            let text = format!("Out {}", 1);
            self.font.draw(window, text_x, text_y, &text, 2.);

            let text = format!("In {}%", 2);
            self.font.draw(window, text_x, text_y_2, &text, 2.);

            let text = format!("{}-{:02}", 3, 4);
            let text_w = self.font.width(&text);
            let text_x = SCREEN_WIDTH - text_w - SCALE as f32;
            self.font.draw(window, text_x, text_y_2, &text, 2.);

            // Button highlights.
            let skill_top: f32 = SCREEN_HEIGHT - bar_height;
            if mouse_pos.y >= skill_top && mouse_pos.x <= SKILL_WIDTH * SKILL_PANEL_SCALE as f32 * SKILL_BUTTONS as f32 {
                let index = mouse_pos.x as isize / (SKILL_WIDTH as isize * SKILL_PANEL_SCALE as isize);
                let alpha: f32 = if mouse[MouseButton::Left].is_down() { 0.4 } else { 0.2 };
                let x: f32 = ((index as f32 * SKILL_WIDTH) + 1.) * SKILL_PANEL_SCALE as f32;
                window.draw_ex(
                    &Rectangle::new((x, skill_top), ((SKILL_WIDTH - 1.) * SKILL_PANEL_SCALE as f32, (SKILL_HEIGHT - 1.) * SKILL_PANEL_SCALE as f32)),
                    Col(Color { r: 1., g: 1., b: 1., a: alpha }),
                    Transform::IDENTITY,
                    3);
            }

            // Skill selection box.
            window.draw_ex( // Left side of box.
                &Rectangle::new(((self.selected_skill_index + 2) as f32 * SKILL_WIDTH * SKILL_PANEL_SCALE as f32, GAME_AREA_HEIGHT),
                    (SKILL_PANEL_SCALE as f32, SKILL_HEIGHT * SKILL_PANEL_SCALE as f32)),
                Col(Color { r: 1., g: 1., b: 1., a: 1. }),
                Transform::IDENTITY,
                4);
            window.draw_ex( // Right side.
                &Rectangle::new(((self.selected_skill_index + 3) as f32 * SKILL_WIDTH * SKILL_PANEL_SCALE as f32, GAME_AREA_HEIGHT),
                    (SKILL_PANEL_SCALE as f32, SKILL_HEIGHT * SKILL_PANEL_SCALE as f32)),
                Col(Color { r: 1., g: 1., b: 1., a: 1. }),
                Transform::IDENTITY,
                4);
            window.draw_ex( // Top.
                &Rectangle::new(((self.selected_skill_index + 2) as f32 * SKILL_WIDTH * SKILL_PANEL_SCALE as f32, GAME_AREA_HEIGHT),
                    (SKILL_WIDTH * SKILL_PANEL_SCALE as f32, SKILL_PANEL_SCALE as f32)),
                Col(Color { r: 1., g: 1., b: 1., a: 1. }),
                Transform::IDENTITY,
                4);
            window.draw_ex( // Bottom.
                &Rectangle::new(((self.selected_skill_index + 2) as f32 * SKILL_WIDTH * SKILL_PANEL_SCALE as f32, GAME_AREA_HEIGHT + (SKILL_HEIGHT - 1.) * SKILL_PANEL_SCALE as f32),
                    (SKILL_WIDTH * SKILL_PANEL_SCALE as f32, SKILL_PANEL_SCALE as f32)),
                Col(Color { r: 1., g: 1., b: 1., a: 1. }),
                Transform::IDENTITY,
                4);
        }

        // Level.
        {
            // Each slice is 1 game pixel, which is SCALE points, or SCALE*2 retina px.
            let displayed_slices: isize = SCREEN_WIDTH as isize / SCALE as isize;
            for screen_slice in 0..displayed_slices {
                let game_slice = self.scroll_offset_x + screen_slice;
                if let Some(slice) = self.level_slices.get(&game_slice) {                    
                    window.draw_ex(&Rectangle::new((screen_slice as f32 * SCALE as f32, 0), (slice.area().size.x / 2., slice.area().size.y / 2.)), Img(slice), Transform::IDENTITY, 3);
                }
            }
            // let size = self.level_image.area().size;
            // let w: f32 = size.x / 2.;
            // let h: f32 = size.y / 2.;
            // let x: f32 = ((SCREEN_HEIGHT - w)/2.).round();
            // let y: f32 = 0.;
            // window.draw_ex(&Rectangle::new((x, y), (w, h)), Img(&self.level_image), Transform::IDENTITY, 1);
        }

        // Objects.
        {
            let ground = &self.game.grounds[&(self.level.globals.normal_graphic_set as i32)];
            for object in self.level.objects.iter() {
                let screen_game_x = object.x - self.scroll_offset_x as i32;
                let screen_x_pt = screen_game_x as f32 * SCALE as f32;
                let screen_y_pt = object.y as f32 * SCALE as f32;
                // TODO skip if offscreen.
                let animation = &ground.object_sprites[&(object.obj_id as i32)];
                let mut current_frame: usize = self.object_animation_frames[&object.obj_id];
                if object.obj_id == OBJECT_ID_START {
                    let last_frame = animation.frames.len() - 1;
                    match self.state {
                        InGameState::CountdownToOpenDoor(_) => current_frame = 1, // 1=closed.
                        InGameState::OpeningDoor(frame) => current_frame = if frame > last_frame { 0 } else { frame },
                        InGameState::Playing => current_frame = 0, // 0 is open.
                    }
                }
                let frames = &self.objects_prescaled[&object.obj_id];
                let frame = &frames[current_frame];
                let size = frame.area().size;
                // TODO honour the options eg upside down.
                window.draw_ex(&Rectangle::new((screen_x_pt, screen_y_pt), (size.x / 2., size.y / 2.)), Img(frame), Transform::IDENTITY, 4);
            }
        }

        // Lemmings.
        {
            for lemming in self.lemmings.iter() {
                // TODO skip if offscreen.
                let action = lemming.action.clone(); // <- weirdly required by rust.
                let frames = self.scaled_lemming_animations.frames_for_action(action, lemming.is_facing_right);
                let frame = &frames[lemming.animation_frame % frames.len()];
                let size = frame.area().size;
                let screen_game_x = lemming.x - self.scroll_offset_x;
                let screen_x_pt = screen_game_x as f32 * SCALE as f32;
                let screen_y_pt = lemming.y as f32 * SCALE as f32;
                window.draw_ex(
                    &Rectangle::new(((screen_x_pt - size.x/4.).round(), (screen_y_pt - size.y/4.).round()), (size.x/2., size.y/2.)),
                    Img(&frame),
                    Transform::IDENTITY,
                    4);
                // match lemming.action {
                //     LemmingAction::Normal => {
                //         let frames = if lemming.is_facing_right { &self.scaled_lemming_animations.walking_right } else { &self.scaled_lemming_animations.walking_left };
                //     },
                //     LemmingAction::Climbing => {},
                //     LemmingAction::Parachuting => {},
                //     LemmingAction::Exploding => {},
                //     LemmingAction::Blocking => {},
                //     LemmingAction::Building => {},
                //     LemmingAction::Bashing => {},
                //     LemmingAction::DiggingDiagonally => {},
                //     LemmingAction::DiggingDown => {},
                //     LemmingAction::Jumping => {},
                //     LemmingAction::Falling => {
                //         let frames = if lemming.is_facing_right { &self.scaled_lemming_animations.falling_right } else { &self.scaled_lemming_animations.falling_left };
                //         let frame = &frames[lemming.animation_frame % frames.len()];
                //         let size = frame.area().size;
                //         let screen_game_x = lemming.x - self.scroll_offset_x;
                //         let screen_x_pt = screen_game_x as f32 * SCALE as f32;
                //         let screen_y_pt = lemming.y as f32 * SCALE as f32;
                //         window.draw_ex(
                //             &Rectangle::new(((screen_x_pt - size.x/4.).round(), (screen_y_pt - size.y/4.).round()), (size.x/2., size.y/2.)),
                //             Img(&frame),
                //             Transform::IDENTITY,
                //             4);
                //     },
                //     LemmingAction::Drowning => {},
                //     LemmingAction::Exiting => {},
                // }
            }
        }

        // Mouse hovering over any lemming.
        {
            if mouse_pos.y < GAME_AREA_HEIGHT {
                if let Some(lemming_index) = self.closest_lemming_to_mouse(&mouse_pos) {
                    let lemming = &self.lemmings[lemming_index as usize];
                    let screen_x = self.screen_x_from_game_x(lemming.x);
                    let screen_y = self.screen_y_from_game_y(lemming.y);

                    window.draw_ex( // Line above.
                        &Rectangle::new((screen_x, screen_y - 10. * SCALE as f32),
                            (SCALE as f32, 4. * SCALE as f32)),
                        Col(Color { r: 1., g: 1., b: 1., a: 1. }),
                        Transform::IDENTITY,
                        5);
                    window.draw_ex( // Line below.
                        &Rectangle::new((screen_x, screen_y + 10. * SCALE as f32),
                            (SCALE as f32, -4. * SCALE as f32)),
                        Col(Color { r: 1., g: 1., b: 1., a: 1. }),
                        Transform::IDENTITY,
                        5);
                    window.draw_ex( // Line left.
                        &Rectangle::new((screen_x - 10. * SCALE as f32, screen_y),
                            (4. * SCALE as f32,  SCALE as f32)),
                        Col(Color { r: 1., g: 1., b: 1., a: 1. }),
                        Transform::IDENTITY,
                        5);
                    window.draw_ex( // Line right.
                        &Rectangle::new((screen_x + 10. * SCALE as f32, screen_y),
                            (-4. * SCALE as f32,  SCALE as f32)),
                        Col(Color { r: 1., g: 1., b: 1., a: 1. }),
                        Transform::IDENTITY,
                        5);

                    // TODO show status up the top or something.
                }
            }
        }

        // Scroll highlights.
        {
            if mouse_pos.y < GAME_AREA_HEIGHT {
                if mouse_pos.x < AUTO_SCROLL_FAST_MARGIN * SCALE as f32 {
                    window.draw_ex(
                        &Rectangle::new((0, 0), (AUTO_SCROLL_FAST_MARGIN * SCALE as f32, GAME_AREA_HEIGHT)),
                        Col(Color { r: 1., g: 1., b: 1., a: 0.2 }),
                        Transform::IDENTITY,
                        6);
                } else if mouse_pos.x > SCREEN_WIDTH - AUTO_SCROLL_FAST_MARGIN * SCALE as f32 {
                    window.draw_ex(
                        &Rectangle::new((SCREEN_WIDTH - AUTO_SCROLL_FAST_MARGIN * SCALE as f32, 0), (AUTO_SCROLL_FAST_MARGIN * SCALE as f32, GAME_AREA_HEIGHT)),
                        Col(Color { r: 1., g: 1., b: 1., a: 0.2 }),
                        Transform::IDENTITY,
                        6);
                } else if mouse_pos.x < AUTO_SCROLL_SLOW_MARGIN * SCALE as f32 {
                    window.draw_ex(
                        &Rectangle::new((0, 0), (AUTO_SCROLL_SLOW_MARGIN * SCALE as f32, GAME_AREA_HEIGHT)),
                        Col(Color { r: 1., g: 1., b: 1., a: 0.1 }),
                        Transform::IDENTITY,
                        6);
                } else if mouse_pos.x > SCREEN_WIDTH - AUTO_SCROLL_SLOW_MARGIN * SCALE as f32 {
                    window.draw_ex(
                        &Rectangle::new((SCREEN_WIDTH - AUTO_SCROLL_SLOW_MARGIN * SCALE as f32, 0), (AUTO_SCROLL_SLOW_MARGIN * SCALE as f32, GAME_AREA_HEIGHT)),
                        Col(Color { r: 1., g: 1., b: 1., a: 0.1 }),
                        Transform::IDENTITY,
                        6);
                }
            }
        }
        Ok(())
    }

    fn next_scene(&mut self) -> Result<Option<Box<dyn Scene>>> {
        // let selected_game = self.selected_game.take(); // Transfer ownership from the ivar.
        // if let Some(selected_game) = selected_game {
        //     let skill_selection = SkillSelection::new(selected_game, self.background.clone())?;
        //     Ok(Some(Box::new(skill_selection)))
        // } else {
            Ok(None)
        // }
    }

}

impl RenderedLevel {
    fn is_clear_under_lemming(&self, x: isize, y: isize) -> bool {
        return self.is_clear(x-2, y+5, 4, 1);
    }
    fn is_clear_to_right_of_lemming(&self, x: isize, y: isize) -> bool {
        return self.is_clear(x+3, y-4, 1, 9);
    }
    fn is_clear_to_left_of_lemming(&self, x: isize, y: isize) -> bool {
        return self.is_clear(x-2, y-4, 1, 9);
    }
    fn is_clear(&self, x: isize, y: isize, w: isize, h: isize) -> bool {
        let min_x = x - self.size.min_x;
        let max_x = min_x + w;
        for myy in y..y+h {            
            if myy<0 || myy>=self.image.height as isize {
                continue; // Skip this row if out of bounds.
            }
            let row = myy as usize * self.image.width;
            for myx in min_x..max_x {                
                if myx < 0 || myx>=self.image.width as isize {
                    continue; // Skip this column if out of bounds.
                }
                if self.image.bitmap[row + myx as usize] != level_renderer::LEVEL_BACKGROUND {
                    return false; // Exit as soon as it finds its first non-clear pixel.
                }
            }
        }
        return true;
    }
}

// This isn't just a helper method on LevelScene because then it'd have to borrow the whole level which doesn't
// suit the callsite which is already borrowing self.
fn mask_area(rendered_level: &mut RenderedLevel, level_scaled_image: &mut Image, level_slices: &mut SliceMap, game_x: isize, game_y: isize, mask: &[u8], mask_w: isize, mask_h: isize) {
    let mut mask_offset: usize = 0;
    for mask_y in 0..mask_h {
        for mask_x in 0..mask_w {
            if mask[mask_offset] != 0 {
                // Remove a pixel from the unscaled rendered_level.
                {
                    let x = mask_x + game_x - rendered_level.size.min_x;
                    let y = mask_y + game_y;
                    if 0<=x && x<rendered_level.image.width as isize && 0<=y && y<rendered_level.image.height as isize {
                        let offset: usize = y as usize * rendered_level.image.width + x as usize;
                        rendered_level.image.bitmap[offset] = level_renderer::LEVEL_BACKGROUND;
                    }
                }

                // Remove a scaled block from the scaled image.
                {
                    let scale = SCALE as isize * 2;
                    let origin_x = (mask_x + game_x - rendered_level.size.min_x) * scale;
                    let origin_y = (mask_y + game_y) * scale;
                    for y in origin_y..(origin_y + scale) {
                        for x in origin_x..(origin_x + scale) {
                            if 0<=x && x<level_scaled_image.width as isize && 0<=y && y<level_scaled_image.height as isize {
                                let offset: usize = y as usize * level_scaled_image.width + x as usize;
                                level_scaled_image.bitmap[offset] = level_renderer::LEVEL_BACKGROUND;
                            }
                        }
                    }
                }
            }
            mask_offset += 1;
        }
    }

    // Re-slice.
    for mask_x in 0..mask_w {
        let slice_game_x = game_x + mask_x;
        let image_x = slice_game_x - rendered_level.size.min_x;
        let slice = slice(&level_scaled_image.bitmap, level_scaled_image.width, level_scaled_image.height, image_x as usize).unwrap();
        level_slices.insert(slice_game_x, slice);
    }
}

fn mask_dig(rendered_level: &mut RenderedLevel, level_scaled_image: &mut Image, level_slices: &mut SliceMap, game_x: isize, game_y: isize, mask_w: isize) {
    for mask_x in 0..mask_w {
        // Remove a pixel from the unscaled rendered_level.
        {
            let x = mask_x + game_x - rendered_level.size.min_x;
            if 0<=x && x<rendered_level.image.width as isize && 0<=game_y && game_y<rendered_level.image.height as isize {
                let offset: usize = game_y as usize * rendered_level.image.width + x as usize;
                rendered_level.image.bitmap[offset] = level_renderer::LEVEL_BACKGROUND;
            }
        }

        // Remove a scaled block from the scaled image.
        {
            let scale = SCALE as isize * 2;
            let origin_x = (mask_x + game_x - rendered_level.size.min_x) * scale;
            let origin_y = game_y * scale;
            for y in origin_y..(origin_y + scale) {
                for x in origin_x..(origin_x + scale) {
                    if 0<=x && x<level_scaled_image.width as isize && 0<=y && y<level_scaled_image.height as isize {
                        let offset: usize = y as usize * level_scaled_image.width + x as usize;
                        level_scaled_image.bitmap[offset] = level_renderer::LEVEL_BACKGROUND;
                    }
                }
            }
        }
    }

    // Re-slice.
    for mask_x in 0..mask_w {
        let slice_game_x = game_x + mask_x;
        let image_x = slice_game_x - rendered_level.size.min_x;
        let slice = slice(&level_scaled_image.bitmap, level_scaled_image.width, level_scaled_image.height, image_x as usize).unwrap();
        level_slices.insert(slice_game_x, slice);
    }
}

// Creates a 1-game-px-wide slice of the map.
// image_x is relative to the rendered_level image, in game pixels.
// Bitmap is the fully scaled image.
fn slice(bitmap: &[u32], w: usize, h: usize, image_x: usize) -> Result<QSImage> {
    let effective_scale: usize = SCALE as usize * 2;
    let pixels_capacity: usize = effective_scale * h;
    let mut slice: Vec<u32> = Vec::with_capacity(pixels_capacity);
    let mut source_offset: usize = image_x * effective_scale;
    let stride: usize = w - effective_scale;
    for _pixel_y in 0..h { // With any luck, these nested loops will be vectorised by the compiler into something fast.
        for _pixel_x in 0..effective_scale {
            slice.push(bitmap[source_offset]);
            source_offset += 1;
        }
        source_offset += stride;
    }
    let slice_image = Image {
        bitmap: slice,
        width: effective_scale,
        height: h,
    };
    qs_image_from_lemmings_image_no_scale(&slice_image)
    // let game_x: isize = image_x as isize + render.size.min_x;
}