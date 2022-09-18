use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::render::render_resource::Extent3d;
use bevy::utils::HashMap;
use crate::{GameTextures, GameState, TEXTURE_SCALE, SCALE, POINT_SIZE};
use crate::lemmings::models::Game;
use crate::level_preview::LevelSelectionResource;
use crate::lemmings::level_renderer;
use crate::helpers::{multi_scale, u32_to_rgba_u8};
use crate::helpers::{make_image_from_bitmap, make_atlas_from_animation};

pub struct InGamePlugin;

impl Plugin for InGamePlugin {
	fn build(&self, app: &mut App) {
		app.add_system_set(
			SystemSet::on_enter(GameState::InGame)
				.with_system(enter)
		);
		app.add_system_set(
			SystemSet::on_update(GameState::InGame)
				.with_system(scroll)
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
struct MapContainerComponent; // Controls the x/y scroll of the map.

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
    pub x: usize,
    pub width: usize,
    pub height: usize,
}

// N pixels in the bitmap, not display points or original lemmings pixels.
// Probably a good number would be the size of a lemming (16px) x scale (12), so each slice is as big as a lemming.
const SLICE_WIDTH: usize = 192;

// Slice a map into pieces of N width.
fn slice(image: &[u32], width: usize, height: usize) -> Vec<SliceWithoutHandle> {
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
        slices.push(SliceWithoutHandle{bitmap: slice_bitmap, x: offset_x, width: this_width, height});
        offset_x += SLICE_WIDTH;
    }
    slices
}

struct Slice {
    pub bitmap: Vec<u32>,
    pub x: usize,
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

/// Scroll left and right if your mouse is at the edge.
fn scroll(
    windows: Res<Windows>,
    mut query: Query<&mut Transform, With<MapContainerComponent>>,
) {
    if let Some(window) = windows.iter().next() {
        if let Some(position) = window.cursor_position() {
            let delta: isize;
            if position.x < window.width() * 0.05 {
                delta = 1;
            } else if position.x > window.width() * 0.95 {
                delta = -1;
            } else {
                delta = 0;
            }
            if delta != 0 {
                for mut transform in query.iter_mut() {
                    transform.translation.x += (delta as f32 * window.width() * 0.005).round();// = Vec3::new(slice.x as f32 * TEXTURE_SCALE, level_offset_y, 1.),
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
	mut images: ResMut<Assets<Image>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
	windows: Res<Windows>,
) {
	if let Some(window) = windows.iter().next() {
		if let Some(level) = game.level_named(&level_selection.level_name) {
            // Spawn the level terrain (not objects).
			let render = level_renderer::render(level, &game.grounds, &game.specials, false);
            let level_offset_y = window.height() / 2. - render.image.height as f32 * POINT_SIZE / 2.;
            let scaled = multi_scale(&render.image.bitmap, render.image.width, render.image.height, false);
            let slices_raw = slice(&scaled, render.image.width * SCALE, render.image.height * SCALE);
            let slices = send_slices_to_bevy(slices_raw, &mut images);
            commands
                .spawn_bundle(SpatialBundle{
                    transform: Transform::from_xyz(-1000., level_offset_y, 1.),
                    ..default()
                }).with_children(|parent| {
                    for slice in &slices {
                        parent.spawn_bundle(SpriteBundle{
                            transform: Transform{
                                // TODO counter the x-centering.
                                translation: Vec3::new(slice.x as f32 * TEXTURE_SCALE, 0., 1.),
                                scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
                                ..default()
                            },
                            texture: slice.texture.clone(),
                            ..default()
                        });
                    }
                })
				.insert(InGameComponent)
                .insert(MapContainerComponent);

            // Scale and bevy-ify the ground's objects.
            pub enum AnimationOrImageHandle {
                Animation(Handle<TextureAtlas>),
                Image(Handle<Image>),
            }
            let mut object_handles = HashMap::<i32, AnimationOrImageHandle>::new();
            let ground = &game.grounds[&(level.globals.normal_graphic_set as i32)];
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

            // Spawn level objects.
            // for object in &level.objects {
            //     let sprite = &ground.object_sprites[&(object.obj_id as i32)];
            //     draw(&sprite.frames[0],
            //         sprite.width as isize, sprite.height as isize,
            //         object.x as isize - size.min_x, object.y as isize,
            //         &mut bitmap,
            //         width as isize, height as isize,
            //         object.modifier.is_do_not_overwrite_existing_terrain(),
            //         object.is_upside_down,
            //         false,
            //         object.modifier.is_must_have_terrain_underneath_to_be_visible());
    
            // }

            // Skill bundle.
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
