use bevy::{prelude::*, render::render_resource::{Extent3d}};
use crate::lemmings::models::Game;
use crate::lemmings_to_bevy::image_doctor::*;
use crate::helpers::{make_image, make_atlas_from_animation};

pub struct LoadLemmingsTexturesPlugin;

impl Plugin for LoadLemmingsTexturesPlugin {
	fn build(&self, app: &mut App) {
        app.add_startup_system(load_lemmings_textures_startup.in_base_set(StartupSet::PreStartup));
	}
}

fn load_lemmings_textures_startup(
    game: Res<Game>,
	mut commands: Commands,
	mut images: ResMut<Assets<Image>>,
	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // Some of them need doctoring a bit.
    let background = doctor_clear_to_black(&game.main.main_menu.background);
    let f1 = doctor_f1(&game.main.main_menu.f1);
    let f2 = doctor_f2(&game.main.main_menu.f2);
    let f3 = doctor_f3(&game.main.main_menu.f3);
    let f4 = doctor_f4(&game.main.main_menu.f4);
    let level_rating = doctor_level_rating(&game.main.main_menu.level_rating);
    let exit_to_dos = doctor_exit_to_dos(&game.main.main_menu.exit_to_dos);
    let mayhem = doctor_skill(&game.main.main_menu.mayhem);
    let taxing = doctor_skill(&game.main.main_menu.taxing);
    let tricky = doctor_skill(&game.main.main_menu.tricky);
    let fun = doctor_skill(&game.main.main_menu.fun);

    // White for solid colours eg fadeouts.
    let white_data: Vec<u8> = vec![0xff, 0xff, 0xff, 0xff];
    let white_image = Image::new(Extent3d{width: 1, height: 1, depth_or_array_layers: 1},
        bevy::render::render_resource::TextureDimension::D2,
        white_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb);
    let white = images.add(white_image);

    let skill_number_digits = SkillNumberDigits::make_images(&game.main.skill_number_digits, &mut images);

    // For inspecting the images conveniently:
    // {
    //     let data = crate::lemmings::png::png_data(game.main.main_menu.exit_to_dos.width as u32, game.main.main_menu.exit_to_dos.height as u32, &game.main.main_menu.exit_to_dos.bitmap);
    //     std::fs::write("exit_to_dos.original.png", &data).unwrap();
    // }
    // {
    //     let data = crate::lemmings::png::png_data(exit_to_dos.width as u32, exit_to_dos.height as u32, &exit_to_dos.bitmap);
    //     std::fs::write("exit_to_dos.doctored.png", &data).unwrap();
    // }

	let game_textures = GameTextures {
        // Menu:
        background: make_image(&background, &mut images, false),
        logo: make_image(&game.main.main_menu.logo, &mut images, true),
        f1: make_image(&f1, &mut images, true),
        f2: make_image(&f2, &mut images, true),
        f3: make_image(&f3, &mut images, true),
        f4_settings: make_image(&f4, &mut images, true),
        level_rating: make_image(&level_rating, &mut images, true),
        exit_to_dos: make_image(&exit_to_dos, &mut images, true),
        music_note: make_image(&game.main.main_menu.music_note, &mut images, true),
        fx: make_image(&game.main.main_menu.fx, &mut images, true),
        blink1: make_atlas_from_animation(&game.main.main_menu.blink1, &mut images, &mut texture_atlases, false),
        blink2: make_atlas_from_animation(&game.main.main_menu.blink2, &mut images, &mut texture_atlases, false),
        blink3: make_atlas_from_animation(&game.main.main_menu.blink3, &mut images, &mut texture_atlases, false),
        blink4: make_atlas_from_animation(&game.main.main_menu.blink4, &mut images, &mut texture_atlases, false),
        blink5: make_atlas_from_animation(&game.main.main_menu.blink5, &mut images, &mut texture_atlases, false),
        blink6: make_atlas_from_animation(&game.main.main_menu.blink6, &mut images, &mut texture_atlases, false),
        blink7: make_atlas_from_animation(&game.main.main_menu.blink7, &mut images, &mut texture_atlases, false),
        left_scroller: make_atlas_from_animation(&game.main.main_menu.left_scroller, &mut images, &mut texture_atlases, true),
        right_scroller: make_atlas_from_animation(&game.main.main_menu.right_scroller, &mut images, &mut texture_atlases, true),
        reel: make_image(&game.main.main_menu.reel, &mut images, true),
        mayhem: make_image(&mayhem, &mut images, true),
        taxing: make_image(&taxing, &mut images, true),
        tricky: make_image(&tricky, &mut images, true),
        fun: make_image(&fun, &mut images, true),
        menu_font: make_atlas_from_animation(&game.main.main_menu.menu_font, &mut images, &mut texture_atlases, true),
    
        // Lemmings:
        walking_right: make_atlas_from_animation(&game.main.lemming_animations.walking_right, &mut images, &mut texture_atlases, true),
        jumping_right: make_atlas_from_animation(&game.main.lemming_animations.jumping_right, &mut images, &mut texture_atlases, true),
        walking_left: make_atlas_from_animation(&game.main.lemming_animations.walking_left, &mut images, &mut texture_atlases, true),
        jumping_left: make_atlas_from_animation(&game.main.lemming_animations.jumping_left, &mut images, &mut texture_atlases, true),
        digging: make_atlas_from_animation(&game.main.lemming_animations.digging, &mut images, &mut texture_atlases, true),
        climbing_right: make_atlas_from_animation(&game.main.lemming_animations.climbing_right, &mut images, &mut texture_atlases, true),
        climbing_left: make_atlas_from_animation(&game.main.lemming_animations.climbing_left, &mut images, &mut texture_atlases, true),
        drowning: make_atlas_from_animation(&game.main.lemming_animations.drowning, &mut images, &mut texture_atlases, true),
        post_climb_right: make_atlas_from_animation(&game.main.lemming_animations.post_climb_right, &mut images, &mut texture_atlases, true),
        post_climb_left: make_atlas_from_animation(&game.main.lemming_animations.post_climb_left, &mut images, &mut texture_atlases, true),
        brick_laying_right: make_atlas_from_animation(&game.main.lemming_animations.brick_laying_right, &mut images, &mut texture_atlases, true),
        brick_laying_left: make_atlas_from_animation(&game.main.lemming_animations.brick_laying_left, &mut images, &mut texture_atlases, true),
        bashing_right: make_atlas_from_animation(&game.main.lemming_animations.bashing_right, &mut images, &mut texture_atlases, true),
        bashing_left: make_atlas_from_animation(&game.main.lemming_animations.bashing_left, &mut images, &mut texture_atlases, true),
        mining_right: make_atlas_from_animation(&game.main.lemming_animations.mining_right, &mut images, &mut texture_atlases, true),
        mining_left: make_atlas_from_animation(&game.main.lemming_animations.mining_left, &mut images, &mut texture_atlases, true),
        falling_right: make_atlas_from_animation(&game.main.lemming_animations.falling_right, &mut images, &mut texture_atlases, true),
        falling_left: make_atlas_from_animation(&game.main.lemming_animations.falling_left, &mut images, &mut texture_atlases, true),
        pre_umbrella_right: make_atlas_from_animation(&game.main.lemming_animations.pre_umbrella_right, &mut images, &mut texture_atlases, true),
        umbrella_right: make_atlas_from_animation(&game.main.lemming_animations.umbrella_right, &mut images, &mut texture_atlases, true),
        pre_umbrella_left: make_atlas_from_animation(&game.main.lemming_animations.pre_umbrella_left, &mut images, &mut texture_atlases, true),
        umbrella_left: make_atlas_from_animation(&game.main.lemming_animations.umbrella_left, &mut images, &mut texture_atlases, true),
        splatting: make_atlas_from_animation(&game.main.lemming_animations.splatting, &mut images, &mut texture_atlases, true),
        exiting: make_atlas_from_animation(&game.main.lemming_animations.exiting, &mut images, &mut texture_atlases, true),
        fried: make_atlas_from_animation(&game.main.lemming_animations.fried, &mut images, &mut texture_atlases, true),
        blocking: make_atlas_from_animation(&game.main.lemming_animations.blocking, &mut images, &mut texture_atlases, true),
        shrugging_right: make_atlas_from_animation(&game.main.lemming_animations.shrugging_right, &mut images, &mut texture_atlases, true), // Builder running out of bricks.
        shrugging_left: make_atlas_from_animation(&game.main.lemming_animations.shrugging_left, &mut images, &mut texture_atlases, true),
        oh_no_ing: make_atlas_from_animation(&game.main.lemming_animations.oh_no_ing, &mut images, &mut texture_atlases, true),
        explosion: make_atlas_from_animation(&game.main.lemming_animations.explosion, &mut images, &mut texture_atlases, true),

        walking_right_count: game.main.lemming_animations.walking_right.frames.len(),
        jumping_right_count: game.main.lemming_animations.jumping_right.frames.len(),
        walking_left_count: game.main.lemming_animations.walking_left.frames.len(),
        jumping_left_count: game.main.lemming_animations.jumping_left.frames.len(),
        digging_count: game.main.lemming_animations.digging.frames.len(),
        climbing_right_count: game.main.lemming_animations.climbing_right.frames.len(),
        climbing_left_count: game.main.lemming_animations.climbing_left.frames.len(),
        drowning_count: game.main.lemming_animations.drowning.frames.len(),
        post_climb_right_count: game.main.lemming_animations.post_climb_right.frames.len(),
        post_climb_left_count: game.main.lemming_animations.post_climb_left.frames.len(),
        brick_laying_right_count: game.main.lemming_animations.brick_laying_right.frames.len(),
        brick_laying_left_count: game.main.lemming_animations.brick_laying_left.frames.len(),
        bashing_right_count: game.main.lemming_animations.bashing_right.frames.len(),
        bashing_left_count: game.main.lemming_animations.bashing_left.frames.len(),
        mining_right_count: game.main.lemming_animations.mining_right.frames.len(),
        mining_left_count: game.main.lemming_animations.mining_left.frames.len(),
        falling_right_count: game.main.lemming_animations.falling_right.frames.len(),
        falling_left_count: game.main.lemming_animations.falling_left.frames.len(),
        pre_umbrella_right_count: game.main.lemming_animations.pre_umbrella_right.frames.len(),
        umbrella_right_count: game.main.lemming_animations.umbrella_right.frames.len(),
        pre_umbrella_left_count: game.main.lemming_animations.pre_umbrella_left.frames.len(),
        umbrella_left_count: game.main.lemming_animations.umbrella_left.frames.len(),
        splatting_count: game.main.lemming_animations.splatting.frames.len(),
        exiting_count: game.main.lemming_animations.exiting.frames.len(),
        fried_count: game.main.lemming_animations.fried.frames.len(),
        blocking_count: game.main.lemming_animations.blocking.frames.len(),
        shrugging_right_count: game.main.lemming_animations.shrugging_right.frames.len(),
        shrugging_left_count: game.main.lemming_animations.shrugging_left.frames.len(),
        oh_no_ing_count: game.main.lemming_animations.oh_no_ing.frames.len(),
        explosion_count: game.main.lemming_animations.explosion.frames.len(),

        skill_panel: make_image(&game.main.skill_panel, &mut images, true),
        skill_selection: make_image(&game.main.skill_selection, &mut images, true),
        speed_selection: make_image(&game.main.speed_selection, &mut images, true),
        pause_selection: make_image(&game.main.pause_selection, &mut images, true),
        nuke_selection: make_image(&game.main.nuke_selection, &mut images, true),
        skill_number_digits, 
        
        white,
        mouse_cursor: make_image(&game.main.mouse_cursor, &mut images, true),
        mouse_cursor_hovering: make_image(&game.main.mouse_cursor_hovering, &mut images, true),
	};
	commands.insert_resource(game_textures);
}

pub struct SkillNumberDigits {
    pub left: [Handle<Image>; 10],
    pub right: [Handle<Image>; 10],
}

impl SkillNumberDigits {
    fn make_images(data: &crate::lemmings::models::SkillNumberDigits, images: &mut ResMut<Assets<Image>>) -> SkillNumberDigits {
        // This ugly thing is because a map returns an array of references, not actual images.
        SkillNumberDigits {
            left: [
                make_image(&data.left[0], images, true),
                make_image(&data.left[1], images, true),
                make_image(&data.left[2], images, true),
                make_image(&data.left[3], images, true),
                make_image(&data.left[4], images, true),
                make_image(&data.left[5], images, true),
                make_image(&data.left[6], images, true),
                make_image(&data.left[7], images, true),
                make_image(&data.left[8], images, true),
                make_image(&data.left[9], images, true),
            ],
            right: [
                make_image(&data.right[0], images, true),
                make_image(&data.right[1], images, true),
                make_image(&data.right[2], images, true),
                make_image(&data.right[3], images, true),
                make_image(&data.right[4], images, true),
                make_image(&data.right[5], images, true),
                make_image(&data.right[6], images, true),
                make_image(&data.right[7], images, true),
                make_image(&data.right[8], images, true),
                make_image(&data.right[9], images, true),
            ]
        }
    }

    /// Safely returns a cloned digit handle, even if index is out of 0 to 9.
    pub fn image(&self, is_left: bool, index: isize) -> Handle<Image> {
        if is_left {
            if 0 <= index && index <= 9 {
                return self.left[index as usize].clone()
            } else {
                return self.left[0].clone()
            }
        } else {
            if 0 <= index && index <= 9 {
                return self.right[index as usize].clone()
            } else {
                return self.right[0].clone()
            }
        }
    }
}

#[derive(Resource)]
pub struct GameTextures {
    // Menu:
    pub background: Handle<Image>,
    // pub background_width: usize,
    // pub background_height: usize,
    pub logo: Handle<Image>,
    pub f1: Handle<Image>,
    pub f2: Handle<Image>,
    pub f3: Handle<Image>,
    pub f4_settings: Handle<Image>,
    pub level_rating: Handle<Image>,
    pub exit_to_dos: Handle<Image>,
    pub music_note: Handle<Image>,
    pub fx: Handle<Image>,
    pub blink1: Handle<TextureAtlas>,
    pub blink2: Handle<TextureAtlas>,
    pub blink3: Handle<TextureAtlas>,
    pub blink4: Handle<TextureAtlas>,
    pub blink5: Handle<TextureAtlas>,
    pub blink6: Handle<TextureAtlas>,
    pub blink7: Handle<TextureAtlas>,
    pub left_scroller: Handle<TextureAtlas>,
    pub right_scroller: Handle<TextureAtlas>,
    pub reel: Handle<Image>,
    pub mayhem: Handle<Image>,
    pub taxing: Handle<Image>,
    pub tricky: Handle<Image>,
    pub fun: Handle<Image>,
    pub menu_font: Handle<TextureAtlas>, // 16x16, '!'(33) - '~'(126), in ascii order. Not a texture atlas for UI's sake.

    // Lemmings:
    pub walking_right: Handle<TextureAtlas>,
    pub jumping_right: Handle<TextureAtlas>, // Walking up a step 3-6px tall. 1 frame.
    pub walking_left: Handle<TextureAtlas>,
    pub jumping_left: Handle<TextureAtlas>, // 1 frame.
    pub digging: Handle<TextureAtlas>,
    pub climbing_right: Handle<TextureAtlas>,
    pub climbing_left: Handle<TextureAtlas>,
    pub drowning: Handle<TextureAtlas>,
    pub post_climb_right: Handle<TextureAtlas>,
    pub post_climb_left: Handle<TextureAtlas>,
    pub brick_laying_right: Handle<TextureAtlas>,
    pub brick_laying_left: Handle<TextureAtlas>,
    pub bashing_right: Handle<TextureAtlas>,
    pub bashing_left: Handle<TextureAtlas>,
    pub mining_right: Handle<TextureAtlas>,
    pub mining_left: Handle<TextureAtlas>,
    pub falling_right: Handle<TextureAtlas>,
    pub falling_left: Handle<TextureAtlas>,
    pub pre_umbrella_right: Handle<TextureAtlas>,
    pub umbrella_right: Handle<TextureAtlas>,
    pub pre_umbrella_left: Handle<TextureAtlas>,
    pub umbrella_left: Handle<TextureAtlas>,
    pub splatting: Handle<TextureAtlas>,
    pub exiting: Handle<TextureAtlas>,
    pub fried: Handle<TextureAtlas>,
    pub blocking: Handle<TextureAtlas>,
    pub shrugging_right: Handle<TextureAtlas>, // Builder running out of bricks.
    pub shrugging_left: Handle<TextureAtlas>,
    pub oh_no_ing: Handle<TextureAtlas>,
    pub explosion: Handle<TextureAtlas>, // 1 frame.

    // Frame counts.
    pub walking_right_count: usize,
    pub jumping_right_count: usize,
    pub walking_left_count: usize,
    pub jumping_left_count: usize,
    pub digging_count: usize,
    pub climbing_right_count: usize,
    pub climbing_left_count: usize,
    pub drowning_count: usize,
    pub post_climb_right_count: usize,
    pub post_climb_left_count: usize,
    pub brick_laying_right_count: usize,
    pub brick_laying_left_count: usize,
    pub bashing_right_count: usize,
    pub bashing_left_count: usize,
    pub mining_right_count: usize,
    pub mining_left_count: usize,
    pub falling_right_count: usize,
    pub falling_left_count: usize,
    pub pre_umbrella_right_count: usize,
    pub umbrella_right_count: usize,
    pub pre_umbrella_left_count: usize,
    pub umbrella_left_count: usize,
    pub splatting_count: usize,
    pub exiting_count: usize,
    pub fried_count: usize,
    pub blocking_count: usize,
    pub shrugging_right_count: usize,
    pub shrugging_left_count: usize,
    pub oh_no_ing_count: usize,
    pub explosion_count: usize,

    // Ingame:
    pub skill_panel: Handle<Image>,
    pub skill_selection: Handle<Image>, // The indicator.
    pub speed_selection: Handle<Image>,
    pub pause_selection: Handle<Image>,
    pub nuke_selection: Handle<Image>,
    pub skill_number_digits: SkillNumberDigits,

    // Other:
    pub white: Handle<Image>,
    pub mouse_cursor: Handle<Image>,
    pub mouse_cursor_hovering: Handle<Image>,
}
