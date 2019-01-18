extern crate quicksilver;

use quicksilver::{
    Result,
    geom::{Rectangle, Transform},
    graphics::{Background::Img, Background::Col, Color, Image as QSImage, PixelFormat},
    lifecycle::{State, Window},
};

use xbrz;
use crate::lemmings::models::*;

pub const SCREEN_WIDTH: f32 = 1920.;
pub const SCREEN_HEIGHT: f32 = 1080.;
pub const SCALE: u8 = 6;

/// Converts an array of pixels/u32s into a u8 array of r,g,b,a.
pub fn rgba_from_pixels(source: &[u32]) -> Vec<u8> {
    let mut rgba: Vec<u8> = Vec::with_capacity(source.len() * 4);
    for abgr in source.iter() {
        rgba.push(*abgr as u8);
        rgba.push((*abgr >> 8) as u8);
        rgba.push((*abgr >> 16) as u8);
        rgba.push((*abgr >> 24) as u8); 
    }
    return rgba;
}

pub fn qs_image_from_lemmings_image(image: &Image) -> Result<quicksilver::graphics::Image> {
    println!("Loaded {} x {}, scaling", image.width, image.height);
    let scaled = xbrz::scale(SCALE, &image.bitmap, image.width as u32, image.height as u32);
    let rgba = rgba_from_pixels(&scaled);
    return quicksilver::graphics::Image::from_raw(&rgba, image.width as u32 * SCALE as u32, image.height as u32 * SCALE as u32, PixelFormat::RGBA);
}

pub fn qs_animation_from_lemmings_animation(animation: &Animation) -> Result<Vec<QSImage>> {
    let mut qs_frames: Vec<QSImage> = Vec::with_capacity(animation.frames.len());
    for frame in animation.frames.iter() {
        let scaled = xbrz::scale(SCALE, &frame, animation.width as u32, animation.height as u32);
        let rgba = rgba_from_pixels(&scaled);
        let qs_frame = quicksilver::graphics::Image::from_raw(&rgba, animation.width as u32 * SCALE as u32, animation.height as u32 * SCALE as u32, PixelFormat::RGBA)?;
        qs_frames.push(qs_frame);
    }
    Ok(qs_frames)
}

// Fonts

pub struct QSMenuFont {
    pub characters: Vec<QSImage>, // '!'(33) - '~'(126), in ascii order.
}
pub fn qs_font_from_lemmings_menu_font(menu_font: &MenuFont) -> Result<QSMenuFont> {
    let mut characters: Vec<QSImage> = Vec::with_capacity(menu_font.characters.len());
    for character in menu_font.characters.iter() {
        let scaled = xbrz::scale(SCALE, &character.bitmap, character.width as u32, character.height as u32);
        let rgba = rgba_from_pixels(&scaled);
        let qs_image = quicksilver::graphics::Image::from_raw(&rgba, character.width as u32 * SCALE as u32, character.height as u32 * SCALE as u32, PixelFormat::RGBA)?;
        characters.push(qs_image);
    }
    Ok(QSMenuFont { characters })
}
impl QSMenuFont {
    /// Returns the height.
    pub fn draw(&self, window: &mut Window, x: f32, y: f32, string: &str, z: f32) {
        let mut current_x = x;
        for char in string.chars() {
            let c = char as usize;
            if 33 <= c && c <= 126 {
                let index = c - 33;
                let image = &self.characters[index as usize];
                let size = image.area().size;
                window.draw_ex(&Rectangle::new((current_x, y), (size.x/2., size.y/2.)), Img(image), Transform::IDENTITY, z);
                current_x += size.x/2.;
            } else if c == 32 { // Space.
                let size = (&self.characters[0]).area().size;
                current_x += size.x/2.;
            }
        }
    }
}
