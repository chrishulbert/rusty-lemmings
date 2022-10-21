use std::time::Duration;
use std::collections::HashMap;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::render::render_resource::Extent3d;
use crate::{GameTextures, GameState, TEXTURE_SCALE, SCALE, POINT_SIZE, FPS};
use crate::lemmings::models::{Game, ObjectInfo};
use crate::level_preview::LevelSelectionResource;
use crate::lemmings::level_renderer;
use crate::helpers::{multi_scale, u32_to_rgba_u8};
use crate::helpers::{make_image_from_bitmap, make_atlas_from_animation};
use crate::{ORIGINAL_GAME_W, FRAME_DURATION};

const DROP_POINTS_PER_FRAME: f32 = 2.;
const LEMMING_NOMINAL_HEIGHT_HALF: i32 = 5; // Usual height for a lemming sprite in game points. Halved for use later.
const LEMMING_WIDTH_FOR_BASE: f32 = 3.; // How many points under it to check to see if any land exists.

pub struct InGamePlugin;

/// Resource.
struct GameTimer(Timer);

/// Resource.
struct InGameStartCountdown(i32); // Countdown to the entrance opening.
struct InGameDropCountdown(i32); // Countdown between dropping lemmings. -1 if hasn't started yet, or has dropped all lemmings.
struct InGameLemmingsContainerId(Entity); // The entity id of the lemmings container.
struct InGameSlices(Option<Slices>);

impl Plugin for InGamePlugin {
	fn build(&self, app: &mut App) {
        // Instead of timers per entity, we use a global timer so that everyone moves in unison.
        app.insert_resource(GameTimer(Timer::from_seconds(FRAME_DURATION, true)));
        app.insert_resource(InGameStartCountdown(FPS as i32));        
        app.insert_resource(InGameDropCountdown(-1));
        app.insert_resource(InGameLemmingsContainerId(Entity::from_raw(0)));
        app.insert_resource(InGameSlices(None));

		app.add_system_set(
			SystemSet::on_enter(GameState::InGame)
				.with_system(enter)
		);
		app.add_system_set(
			SystemSet::on_update(GameState::InGame)
                .label("tick")
                .with_system(tick)
		);
		app.add_system_set(
			SystemSet::on_update(GameState::InGame)
                .after("tick")
				.with_system(scroll)
                .with_system(update_objects)
                .with_system(do_countdown)
                .with_system(drop_lemmings)
                .with_system(update_lemmings)               
		);
		app.add_system_set(
		    SystemSet::on_exit(GameState::InGame)
		        .with_system(exit)
		);
	}
}

#[derive(Component)]
struct InGameComponent; // Marker component so it can be despawned.

#[derive(Component)]
struct ObjectComponent {
    pub info: ObjectInfo,
}

#[derive(Component)]
struct LemmingComponent {
    is_facing_right: bool,
}

#[derive(Component)]
struct MapContainerComponent { // Controls the x/y scroll of the map.
    pub min_x: f32, // In bevy transform coords.
    pub max_x: f32,
}

fn exit(
    mut commands: Commands,
    menu_components: Query<Entity, With<InGameComponent>>,
) {
    for e in menu_components.iter() {
        commands.entity(e).despawn_recursive();
    }
}

struct SliceWithoutHandle {
    pub bitmap: Vec<u32>,
    pub x: isize, // Scaled-up pixels. This includes the level's min_x.
    pub width: usize,
    pub height: usize,
}

// N pixels in the bitmap, not display points or original lemmings pixels.
// Probably a good number would be the size of a lemming (16px) x scale (12), so each slice is as big as a lemming.
const SLICE_WIDTH: usize = 192;

// Slice a map into pieces of N width.
// The image should be scaled.
fn slice(image: &[u32], width: usize, height: usize, min_x: isize) -> Vec<SliceWithoutHandle> {
    let mut slices = Vec::<SliceWithoutHandle>::with_capacity(width / SLICE_WIDTH + 1);
    let mut offset_x: usize = 0;
    while offset_x < width {
        let remaining_cols = width - offset_x;
        let this_width = std::cmp::min(SLICE_WIDTH, remaining_cols);
        let mut slice_bitmap = Vec::<u32>::with_capacity(this_width * height);
        for y in 0..height {
            let input_offset = y * width + offset_x;
            for x in 0..this_width {
                slice_bitmap.push(image[input_offset + x]);
            }
        }
        slices.push(SliceWithoutHandle{bitmap: slice_bitmap, x: (offset_x as isize) + min_x, width: this_width, height});
        offset_x += SLICE_WIDTH;
    }
    slices
}

struct Slices {
    slices: Vec<Slice>,
    x_to_slice_index_lookup: HashMap<i32, u32>, // eg x_to_slice_index[x] = index in slices.
}

struct Slice {
    pub bitmap: Vec<u32>,
    pub texture: Handle<Image>,

    // The following are in scaled-up pixels:
    pub x: isize, 
    pub width: usize,
    pub height: usize,

    // In game points:
    game_points_x: isize,
    game_points_width: usize,
}

fn convert_slices_to_bevy(in_slices: Vec<SliceWithoutHandle>, images: &mut ResMut<Assets<Image>>) -> Slices {
    let slices: Vec<Slice> = in_slices.into_iter().map(|s| {
        let u8_data = u32_to_rgba_u8(&s.bitmap);
        let image = Image::new(Extent3d{width: s.width as u32, height: s.height as u32, depth_or_array_layers: 1},
            bevy::render::render_resource::TextureDimension::D2,
            u8_data,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb);    
        let texture = images.add(image);
        Slice{
            bitmap: s.bitmap,
            texture,
            x: s.x,
            width: s.width,
            height: s.height,
            game_points_x: s.x / SCALE as isize,
            game_points_width: s.width / SCALE,
        }
    }).collect();

    // Make a lookup so its easy to find the slices.
    // Not using a simple array because the x positions might be negative.
    let mut x_to_slice_index_lookup = HashMap::<i32, u32>::new();
    for (slice_index, slice) in slices.iter().enumerate() {
        for delta in 0..slice.game_points_width {
            x_to_slice_index_lookup.insert(slice.game_points_x as i32 + delta as i32, slice_index as u32);
        }
    }

    Slices {
        slices,
        x_to_slice_index_lookup,
    }
}

// Drop lemmings every now and again.
fn drop_lemmings(
	mut commands: Commands,
	game_textures: Res<GameTextures>,
    timer: Res<GameTimer>,
    mut drop_countdown: ResMut<InGameDropCountdown>,
    query: Query<(&Transform, &ObjectComponent)>,
    lemmings_container_id: Res<InGameLemmingsContainerId>,
) {
    if drop_countdown.0 < 0 { return } // hasn't started yet or is complete.
    if timer.0.just_finished() {
        let new_countdown = drop_countdown.0 - 1;
        if new_countdown <= 0 {
            // For each entrance, drop 1 lemming, until we've dropped enough lemmings.
            for (t, o) in query.iter() {
                let t: &Transform = t;
                let o: &ObjectComponent = o;
                if o.info.is_entrance {

                    // TODO only spawn if we haven't run out of lemmings to spawn, otherwise break the for.
                    spawn_a_lemming(&mut commands, &t.translation, &game_textures, &lemmings_container_id.0);
                }
            }
            drop_countdown.0 = FPS as i32; // Delay until the next drop. TODO change this depending on the selected drop rate.
        } else {
            drop_countdown.0 = new_countdown;
        }
    }
}

// Spawn a lemming right now.
fn spawn_a_lemming(
    commands: &mut Commands,
    entrance: &Vec3,
    game_textures: &Res<GameTextures>,
    lemmings_container_id: &Entity,
) {
    commands.entity(*lemmings_container_id).with_children(|parent| {
        parent.spawn_bundle(SpriteSheetBundle{
            texture_atlas: game_textures.falling_right.clone(),
            transform: Transform{
                scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
                translation: Vec3::new(entrance.x.round(), entrance.y.round(), 0.),
                ..default()
            },
            ..default()
        }).insert(LemmingComponent{is_facing_right: true});
    });
}

fn update_objects(
    timer: Res<GameTimer>,
    start_countdown: Res<InGameStartCountdown>,
    mut drop_countdown: ResMut<InGameDropCountdown>,
    mut query: Query<(
        &mut TextureAtlasSprite,
        &ObjectComponent,
    )>,
) {
    if timer.0.just_finished() {
        for (mut tas, object_unknown) in &mut query {
            let object: &ObjectComponent = object_unknown; // Otherwise RLS can't suggest the type.
            if object.info.is_entrance {
                // Entrance is a special case: has to wait for the start countdown.
                if start_countdown.0 <= 0 {
                    if tas.index > 0 { // Not fully open yet.
                        let new_index = tas.index + 1;
                        if new_index >= object.info.frame_count as usize {
                            tas.index = 0; // Full open now.
                            drop_countdown.0 = (FPS / 2.) as i32; // Wait half a sec to drop.
                        } else {
                            tas.index = new_index;
                        }    
                    }
                }
            } else {
                tas.index = (tas.index + 1) % (object.info.frame_count as usize);
            }
        }
    }
}

fn do_countdown(
    timer: Res<GameTimer>,
    mut start_countdown: ResMut<InGameStartCountdown>,
) {
    if start_countdown.0 > 0 {
        if timer.0.just_finished() {
            start_countdown.0 -= 1;
        }    
    }
}

fn tick(
    time: Res<Time>,
    mut timer: ResMut<GameTimer>,
) {
    timer.0.tick(time.delta());
}

/// Scroll left and right if your mouse is at the edge.
fn scroll(
    time: Res<Time>,
    windows: Res<Windows>,
    mut query: Query<(&mut Transform, &MapContainerComponent)>,
) {
    if let Some(window) = windows.iter().next() {
        if let Some(position) = window.cursor_position() {
            let delta: isize;
            if position.x < window.width() * 0.05 {
                delta = 2;
            } else if position.x < window.width() * 0.1 {
                delta = 1;
            } else if position.x > window.width() * 0.95 {
                delta = -2;
            } else if position.x > window.width() * 0.9 {
                delta = -1;
            } else {
                delta = 0;
            }
            if delta != 0 {
                for (mut transform, container) in query.iter_mut() {
                    let new_x = transform.translation.x + (delta as f32 * time.delta().as_secs_f32() * window.width() * 0.3).round();
                    let clamped_x = new_x.min(container.max_x).max(container.min_x);
                    transform.translation.x = clamped_x;
                }
            }
        }
    }
}

fn enter(
	mut commands: Commands,
	game_textures: Res<GameTextures>,
    level_selection: Res<LevelSelectionResource>,
	game: Res<Game>,
    mut timer: ResMut<GameTimer>,
    mut start_countdown: ResMut<InGameStartCountdown>,
    mut drop_countdown: ResMut<InGameDropCountdown>,
	mut images: ResMut<Assets<Image>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut lemmings_container_id: ResMut<InGameLemmingsContainerId>,
    mut slices_resource: ResMut<InGameSlices>,
	windows: Res<Windows>,
) {
    start_countdown.0 = FPS as i32;
    drop_countdown.0 = -1; // Not dropping yet.
    timer.0.set_elapsed(Duration::ZERO);
	if let Some(window) = windows.iter().next() {
		if let Some(level) = game.level_named(&level_selection.level_name) {
            // Scale and bevy-ify the ground's objects.
            let ground = &game.grounds[&(level.globals.normal_graphic_set as i32)];
            pub enum AnimationOrImageHandle {
                Animation(Handle<TextureAtlas>),
                Image(Handle<Image>),
            }
            let mut object_handles = HashMap::<i32, AnimationOrImageHandle>::new();
            for (index, animation) in &ground.object_sprites {
                let frame_count = animation.frames.len();
                let anim_or_image: AnimationOrImageHandle;
                if frame_count == 0 {
                    continue;
                } else if frame_count == 1 {
                    // Static.
                    let image_handle = make_image_from_bitmap(&animation.frames[0], animation.width, animation.height, &mut images, true);
                    anim_or_image = AnimationOrImageHandle::Image(image_handle);
                } else {
                    // Animation.
                    let atlas_handle = make_atlas_from_animation(animation, &mut images, &mut texture_atlases, true);
                    anim_or_image = AnimationOrImageHandle::Animation(atlas_handle);
                }
                object_handles.insert(index.clone(), anim_or_image);
            }

            // Spawn the level terrain.
			let render = level_renderer::render(level, &game.grounds, &game.specials, false);
            let game_origin_offset_y: f32 = (render.image.height as f32) * POINT_SIZE / 2.; // Y to use for 0 in game coords.
            let level_offset_y = window.height() / 2. - game_origin_offset_y;
            let scaled = multi_scale(&render.image.bitmap, render.image.width, render.image.height, false);
            let slices_raw = slice(&scaled, render.image.width * SCALE, render.image.height * SCALE, render.size.min_x * SCALE as isize);
            let slices = convert_slices_to_bevy(slices_raw, &mut images);
            commands
                .spawn_bundle(SpatialBundle{
                    // TODO for the start X, do we need to account for the current screen width?
                    transform: Transform::from_xyz(-(level.globals.start_screen_xpos as f32 + ORIGINAL_GAME_W as f32 / 2.) * POINT_SIZE, 
                        level_offset_y, 1.),
                    ..default()
                }).with_children(|parent| {
                    // Terrain slices.
                    parent.spawn_bundle(SpatialBundle::default()).with_children(|parent| {
                        for slice in &slices.slices {
                            parent.spawn_bundle(SpriteBundle{
                                transform: Transform{
                                    translation: Vec3::new((slice.x as f32 + (slice.width as f32 / 2.)) * TEXTURE_SCALE, 0., 2.),
                                    scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
                                    ..default()
                                },
                                texture: slice.texture.clone(),
                                ..default()
                            });
                        }
                    });

                    // Spawn level objects.
                    for object in &level.objects {
                        let z_index: f32 = if object.modifier.is_do_not_overwrite_existing_terrain() { 1. } else { 3. };
                        let object_info = &ground.ground.object_info[object.obj_id];
                        if let Some(handle) = object_handles.get(&(object.obj_id as i32)) {
                            let transform = Transform{
                                scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
                                translation: Vec3::new((object.x as f32 + object_info.width as f32 / 2.) * POINT_SIZE,
                                game_origin_offset_y - (object.y as f32 + object_info.height as f32 / 2.) * POINT_SIZE, 
                                z_index),
                                ..default()
                            };
                            let object_component = ObjectComponent{
                                info: object_info.clone(),
                            };
                            match handle {
                                AnimationOrImageHandle::Animation(anim) => {
                                    parent.spawn_bundle(SpriteSheetBundle{
                                        sprite: TextureAtlasSprite { index: object_info.start_animation_frame_index as usize % object_info.frame_count as usize, ..default() },
                                        texture_atlas: anim.clone(),
                                        transform, 
                                        ..default()
                                    })
                                    .insert(object_component);
                                },
                                AnimationOrImageHandle::Image(image) => {
                                    parent.spawn_bundle(SpriteBundle{
                                        texture: image.clone(),
                                        transform, 
                                        ..default()
                                    })
                                    .insert(object_component);
                                },
                            }
                        }    
                    }

                    // Spawn lemmings container.
                    lemmings_container_id.0 = parent.spawn_bundle(SpatialBundle {
                        transform: Transform::from_xyz(0., 0., 4.),
                       ..default() 
                    }).id();
                })
				.insert(InGameComponent)
                .insert(MapContainerComponent{
                    min_x: -render.size.max_x as f32 * POINT_SIZE,
                    max_x: -render.size.min_x as f32 * POINT_SIZE,
                });

            // Keep the slices around.
            slices_resource.0 = Some(slices);

            // Skill panel.
			commands
				.spawn_bundle(SpriteBundle{
                    sprite: Sprite { anchor: Anchor::BottomCenter, ..default() },
					texture: game_textures.skill_panel.clone(),
                    transform: Transform{
                        translation: Vec3::new(0., -window.height() / 2., 10.),
                        scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
                        ..default()
                    },        
                    ..default()
				})
				.insert(InGameComponent);
		}
	}
}

fn update_lemmings(
    mut query: Query<(
        &mut Transform,
        &mut TextureAtlasSprite,
        &mut Handle<TextureAtlas>,
        &LemmingComponent,
    )>,
    timer: Res<GameTimer>,
    slices: Res<InGameSlices>,
    game_textures: Res<GameTextures>,
) {
    if !timer.0.just_finished() { return }

    for (mut t, mut tas, mut ta, l) in query.iter_mut() {
        let mut t: Mut<Transform> = t;
        let mut tas: Mut<TextureAtlasSprite> = tas;
        let l: &LemmingComponent = l;
        let (game_x, game_y) = game_xy_from_translation(&t.translation);
        let bottom_y = game_y + LEMMING_NOMINAL_HEIGHT_HALF;
        let mut did_change_texture = false;
        let x_delta_from_direction_faced: f32 = if l.is_facing_right { 1. } else { -1. };
        // Check if there's any ground under this lemming.
        if is_there_ground_at_xy(game_x, bottom_y, slices.0.as_ref()) {
            let direction: f32;
            if l.is_facing_right {
                if ta.id != game_textures.walking_right.id {
                    *ta = game_textures.walking_right.clone();
                    tas.index = 0;
                    did_change_texture = true;
                }
            } else {
                if ta.id != game_textures.walking_left.id {
                    *ta = game_textures.walking_left.clone();
                    tas.index = 0;
                    did_change_texture = true;
                }
            }
            t.translation.x = (t.translation.x + x_delta_from_direction_faced * POINT_SIZE).round();
            // Walk left or right. If there's no ground under it to the side, he can climb down or up no dramas without needing to fall.
        } else {
            // TODO if there was nothing under it, iterate DROP_POINTS_PER_FRAME times.
            if l.is_facing_right {
                if ta.id != game_textures.falling_right.id {
                    *ta = game_textures.falling_right.clone();
                    tas.index = 0;
                    did_change_texture = true;
                }
            } else {
                if ta.id != game_textures.falling_left.id {
                    *ta = game_textures.falling_left.clone();
                    tas.index = 0;
                    did_change_texture = true;
                }
            }
            t.translation.y = (t.translation.y - POINT_SIZE).round(); // Round on changes so we don't accumulate some float error.
        }
        if !did_change_texture {
            tas.index = (tas.index + 1) % 4;
        }
    }
}

fn game_xy_from_translation(translation: &Vec3) -> (i32, i32) {
    // Translation 0 means middle, and + numbers go towards the top of the screen.
    let game_y = level_renderer::LEVEL_HEIGHT as f32 / 2. - translation.y / POINT_SIZE;
    let game_x = translation.x / POINT_SIZE;
    (game_x.round() as i32, game_y.round() as i32)
}

// xy are game points, eg y=0=top.
fn is_there_ground_at_xy(x: i32, y: i32, slices_o: Option<&Slices>) -> bool {
    if let Some(slices) = slices_o {
        if let Some(index) = slices.x_to_slice_index_lookup.get(&x) {
            let slice = &slices.slices[*index as usize];
            let game_points_x_offset = x as isize - slice.game_points_x;
            let scaled_x_offset = game_points_x_offset as usize * SCALE;
            let scaled_y_offset = y as usize * SCALE;
            let offset = scaled_y_offset * slice.width + scaled_x_offset;
            // Check all scaled pixels until ground is found.
            // Exit early if ground is found. This is an optimisation because lemmings will more
            // often than not be on the ground.
            for scale_search_x in 0..SCALE {
                if let Some(rgba) = slice.bitmap.get(offset + scale_search_x) {
                    let a = *rgba as u8;
                    if a > 0 {
                        return true
                    }
                }
            }
            return false
        } else {
            false
        }
    } else {
        false
    }
}
