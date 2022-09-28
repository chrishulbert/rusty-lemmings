use std::time::Duration;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::render::render_resource::Extent3d;
use bevy::utils::HashMap;
use crate::{GameTextures, GameState, TEXTURE_SCALE, SCALE, POINT_SIZE, FPS};
use crate::lemmings::models::{Game, ObjectInfo, Object};
use crate::level_preview::LevelSelectionResource;
use crate::lemmings::level_renderer;
use crate::helpers::{multi_scale, u32_to_rgba_u8};
use crate::helpers::{make_image_from_bitmap, make_atlas_from_animation};
use crate::{ORIGINAL_GAME_W, FRAME_DURATION};

pub struct InGamePlugin;

/// Resource.
struct GameTimer(Timer);

/// Resource.
struct InGameStartCountdown(i32);

impl Plugin for InGamePlugin {
	fn build(&self, app: &mut App) {
        // Instead of timers per entity, we use a global timer so that everyone moves in unison.
        app.insert_resource(GameTimer(Timer::from_seconds(FRAME_DURATION, true)));
        app.insert_resource(InGameStartCountdown(FPS as i32));

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
    pub x: isize,
    pub width: usize,
    pub height: usize,
}

// N pixels in the bitmap, not display points or original lemmings pixels.
// Probably a good number would be the size of a lemming (16px) x scale (12), so each slice is as big as a lemming.
const SLICE_WIDTH: usize = 192;

// Slice a map into pieces of N width.
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

struct Slice {
    pub bitmap: Vec<u32>,
    pub x: isize,
    pub width: usize,
    pub height: usize,
    pub texture: Handle<Image>,
}

fn send_slices_to_bevy(slices: Vec<SliceWithoutHandle>, images: &mut ResMut<Assets<Image>>) -> Vec<Slice> {
    slices.into_iter().map(|s| {
        let u8_data = u32_to_rgba_u8(&s.bitmap);
        let image = Image::new(Extent3d{width: s.width as u32, height: s.height as u32, depth_or_array_layers: 1},
            bevy::render::render_resource::TextureDimension::D2,
            u8_data,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb);    
        let texture = images.add(image);
        Slice{
            bitmap: s.bitmap,
            x: s.x,
            width: s.width,
            height: s.height,
            texture,
        }
    }).collect()
}

fn update_objects(
    timer: Res<GameTimer>,
    mut query: Query<(
        &mut TextureAtlasSprite,
        &ObjectComponent,
    )>,
) {
    if timer.0.just_finished() {
        for (mut tas, object) in &mut query {
            tas.index = (tas.index + 1) % (object.info.frame_count as usize);
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
	mut images: ResMut<Assets<Image>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
	windows: Res<Windows>,
) {
    start_countdown.0 = FPS as i32;
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
            let slices = send_slices_to_bevy(slices_raw, &mut images);
            commands
                .spawn_bundle(SpatialBundle{
                    // TODO for the start X, do we need to account for the current screen width?
                    transform: Transform::from_xyz(-(level.globals.start_screen_xpos as f32 + ORIGINAL_GAME_W as f32 / 2.) * POINT_SIZE, 
                        level_offset_y, 1.),
                    ..default()
                }).with_children(|parent| {
                    // Terrain slices.
                    parent.spawn_bundle(SpatialBundle::default()).with_children(|parent| {
                        for slice in &slices {
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
                })
				.insert(InGameComponent)
                .insert(MapContainerComponent{
                    min_x: -render.size.max_x as f32 * POINT_SIZE,
                    max_x: -render.size.min_x as f32 * POINT_SIZE,
                });

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
