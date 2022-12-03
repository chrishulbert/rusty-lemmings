use bevy::prelude::*;
use bevy::math::Rect;
use bevy::render::render_resource::Extent3d;
use crate::xbrz;
use crate::{SCALE, SCALE_A, SCALE_B};
use crate::lemmings::models::Animation;

/// Convert our lemmings images into bevy compatible ones.
pub fn u32_to_rgba_u8(u32s: &[u32]) -> Vec<u8> {
    let mut u8s = Vec::<u8>::with_capacity(u32s.len() * 4);
    for u in u32s {
        u8s.push((u >> 24) as u8);
        u8s.push((u >> 16) as u8);
        u8s.push((u >> 8) as u8);
        u8s.push(*u as u8);
    }
    u8s
}

fn add_1_margin(image: &[u32], width: usize, height: usize) -> (Vec<u32>, usize, usize) {
    let margin_width = width + 2;
    let margin_height = height + 2;
    let margin_pixels = margin_width * margin_height;
    let mut image_with_margin = Vec::<u32>::with_capacity(margin_pixels);
    image_with_margin.resize(margin_pixels, 0);
    let mut in_offset: usize = 0;
    let mut out_offset: usize = margin_width + 1;
    for _y in 0..height {
        for _x in 0..width {
            image_with_margin[out_offset] = image[in_offset];
            out_offset += 1;
            in_offset += 1;
        }
        out_offset += 2;
    }
    (image_with_margin, margin_width, margin_height)
}

fn remove_scale_margin(image: &[u32], width: usize, height: usize) -> Vec<u32> {
    let smaller_width = width - 2 * SCALE;
    let smaller_height = height - 2 * SCALE;
    let smaller_pixels = smaller_width * smaller_height;
    let mut smaller_image = Vec::<u32>::with_capacity(smaller_pixels);
    smaller_image.resize(smaller_pixels, 0);
    let mut in_offset: usize = width * SCALE + SCALE;
    let mut out_offset: usize = 0;
    for _y in 0..smaller_height {
        for _x in 0..smaller_width {
            smaller_image[out_offset] = image[in_offset];
            out_offset += 1;
            in_offset += 1;
        }
        in_offset += 2 * SCALE;
    }
    smaller_image
}

// Multi-step scale-up.
// should_add_then_remove_margin removes artifacts from sprites (eg not things that are expected to tile) where they don't
// 'round off' near the edge properly.
pub fn multi_scale(image: &[u32], width: usize, height: usize, should_add_then_remove_margin: bool) -> Vec<u32> {
    if should_add_then_remove_margin {
        let (image_with_margin, margin_width, margin_height) = add_1_margin(image, width, height);
        let scaled = multi_scale(&image_with_margin, margin_width, margin_height, false);
        return remove_scale_margin(&scaled, margin_width * SCALE, margin_height * SCALE);
    }
    let bigger = xbrz::scale(SCALE_A as u8, image, width as u32, height as u32);
    xbrz::scale(SCALE_B as u8, &bigger, (width * SCALE_A) as u32, (height * SCALE_A) as u32)
}

pub fn make_image(
    image: &crate::lemmings::models::Image,
    images: &mut ResMut<Assets<Image>>,
    should_add_then_remove_margin: bool,
) -> Handle<Image> {
    make_image_from_bitmap(&image.bitmap, image.width, image.height, images, should_add_then_remove_margin)
}

pub fn make_image_from_bitmap(
    bitmap: &[u32],
    width: usize,
    height: usize,
    images: &mut ResMut<Assets<Image>>,
    should_add_then_remove_margin: bool,
) -> Handle<Image> {
    let scaled = multi_scale(bitmap, width, height, should_add_then_remove_margin);
    let u8_data = u32_to_rgba_u8(&scaled);
    let image = Image::new(Extent3d{width: (width * SCALE) as u32, height: (height * SCALE) as u32, depth_or_array_layers: 1},
        bevy::render::render_resource::TextureDimension::D2,
        u8_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb);
    let image_handle = images.add(image);
    image_handle
}

pub fn make_image_unscaled(
    image: &crate::lemmings::models::Image,
    images: &mut ResMut<Assets<Image>>,
) -> Handle<Image> {
    let u8_data = u32_to_rgba_u8(&image.bitmap);
    let image = Image::new(Extent3d{width: image.width as u32, height: image.height as u32, depth_or_array_layers: 1},
        bevy::render::render_resource::TextureDimension::D2,
        u8_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb);
    let image_handle = images.add(image);
    image_handle
}

// Figure out a neat way to layout the grid.
fn cols_rows_for_frames(frame_count: usize) -> (usize, usize) {
    let lenf = frame_count as f32;
    let cols = lenf.sqrt().round() as usize;
    let divides_perfectly = frame_count % cols == 0;
    let rows = if divides_perfectly { frame_count / cols } else { frame_count / cols + 1};
    (cols, rows)
}

pub fn make_atlas_from_animation(
    animation: &Animation,
    images: &mut ResMut<Assets<Image>>,
	texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    should_add_then_remove_margin: bool,
) -> Handle<TextureAtlas> {
    let (cols, rows) = cols_rows_for_frames(animation.frames.len());
    let scaled_width = animation.width * SCALE;
    let scaled_height = animation.height * SCALE;
    let atlas_width = (scaled_width + 1) * cols - 1; // 1px gap between each.
    let atlas_height = (scaled_height + 1) * rows - 1;
    let atlas_pixels = atlas_width * atlas_height;
    let mut atlas = Vec::<u32>::with_capacity(atlas_pixels);
    atlas.resize(atlas_pixels, 0);
    let mut col: usize = 0;
    let mut row: usize = 0;
    let mut sprite_rects = Vec::<Rect>::with_capacity(animation.frames.len());
    for small_frame in &animation.frames {
        let scaled_frame = multi_scale(small_frame, animation.width, animation.height, should_add_then_remove_margin);
        let start_atlas_x = col * (scaled_width + 1);
        let mut atlas_y = row * (scaled_height + 1);

        sprite_rects.push(Rect { 
            min: Vec2 { x: start_atlas_x as f32, y: atlas_y as f32 },
            max: Vec2 { x: (start_atlas_x + scaled_width) as f32, y: (atlas_y + scaled_height) as f32 } });

        for frame_y in 0..scaled_height {
            let mut atlas_x = start_atlas_x;
            for frame_x in 0..scaled_width {
                atlas[atlas_y * atlas_width + atlas_x] = scaled_frame[frame_y * scaled_width + frame_x];
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
    // Convert it for bevy now.
    let u8_data = u32_to_rgba_u8(&atlas);
    let image = Image::new(Extent3d{width: atlas_width as u32, height: atlas_height as u32, depth_or_array_layers: 1},
        bevy::render::render_resource::TextureDimension::D2,
        u8_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb);
    let image_handle = images.add(image);
    let mut ta = TextureAtlas::new_empty(image_handle, Vec2::new(atlas_width as f32, atlas_height as f32));    
    for rect in sprite_rects {
        ta.add_texture(rect);
    }
    let ta_handle = texture_atlases.add(ta);
    ta_handle
}