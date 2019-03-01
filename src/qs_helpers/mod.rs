extern crate quicksilver;

use quicksilver::{
    Result,
    geom::{Rectangle, Transform},
    graphics::{Background::Img, Image as QSImage, PixelFormat},
    lifecycle::{Window},
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
    qs_font_from_lemmings_menu_font_with_scale(menu_font, SCALE)
}
pub fn qs_font_from_lemmings_menu_font_with_scale(menu_font: &MenuFont, scale: u8) -> Result<QSMenuFont> {
    let mut characters: Vec<QSImage> = Vec::with_capacity(menu_font.characters.len());
    for character in menu_font.characters.iter() {
        let scaled = xbrz::scale(scale, &character.bitmap, character.width as u32, character.height as u32);
        let rgba = rgba_from_pixels(&scaled);
        let qs_image = quicksilver::graphics::Image::from_raw(&rgba, character.width as u32 * scale as u32, character.height as u32 * scale as u32, PixelFormat::RGBA)?;
        characters.push(qs_image);
    }
    Ok(QSMenuFont { characters })
}
impl QSMenuFont {
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
    pub fn width(&self, string: &str) -> f32 {
        return self.characters[0].area().size.x / 2. * string.chars().count() as f32;
    }
}

impl Image {
    fn qs_scaled(&self) -> Result<QSImage> {
        let scaled = xbrz::scale(SCALE, &self.bitmap, self.width as u32, self.height as u32);
        let rgba = rgba_from_pixels(&scaled);
        let qs_image = quicksilver::graphics::Image::from_raw(&rgba, self.width as u32 * SCALE as u32, self.height as u32 * SCALE as u32, PixelFormat::RGBA)?;
        Ok(qs_image)
    }
}
pub struct QSGameFont {
    pub percent: QSImage,
    pub digits: Vec<QSImage>, // 0-9
    pub dash: QSImage,
    pub letters: Vec<QSImage>, // A-Z
}
impl GameFont {
    pub fn qs_font(&self) -> Result<QSGameFont> {
        let percent = self.percent.qs_scaled()?;
        let dash = self.dash.qs_scaled()?;
        let digits: Result<Vec<QSImage>> = self.digits.iter().map(|i| i.qs_scaled()).collect();
        let letters: Result<Vec<QSImage>> = self.letters.iter().map(|i| i.qs_scaled()).collect();
        Ok(QSGameFont{
            percent,
            digits: digits?,
            dash,
            letters: letters?,
        })
    }
}
impl QSGameFont {
    pub fn draw(&self, window: &mut Window, x: f32, y: f32, string: &str, z: f32) {
        let mut current_x = x;
        for char in string.chars() {
            let c = char as usize;
            if c == 37 { // Percent.
                let image = &self.percent;
                let size = image.area().size;
                window.draw_ex(&Rectangle::new((current_x, y), (size.x/2., size.y/2.)), Img(image), Transform::IDENTITY, z);
                current_x += size.x/2.;
            } else if 48 <= c && c <= 57 { // Digit
                let index = c - 48;
                let image = &self.digits[index as usize];
                let size = image.area().size;
                window.draw_ex(&Rectangle::new((current_x, y), (size.x/2., size.y/2.)), Img(image), Transform::IDENTITY, z);
                current_x += size.x/2.;
            } else if 65 <= c && c <= 90 { // A-Z
                let index = c - 65;
                let image = &self.letters[index as usize];
                let size = image.area().size;
                window.draw_ex(&Rectangle::new((current_x, y), (size.x/2., size.y/2.)), Img(image), Transform::IDENTITY, z);
                current_x += size.x/2.;
            } else if 97 <= c && c <= 122 { // a-z
                let index = c - 97;
                let image = &self.letters[index as usize];
                let size = image.area().size;
                window.draw_ex(&Rectangle::new((current_x, y), (size.x/2., size.y/2.)), Img(image), Transform::IDENTITY, z);
                current_x += size.x/2.;
            } else if c == 32 { // Space
                let size = self.percent.area().size;
                current_x += size.x/2.;
            } else { // Anything else, draw a dash.
                let image = &self.dash;
                let size = image.area().size;
                window.draw_ex(&Rectangle::new((current_x, y), (size.x/2., size.y/2.)), Img(image), Transform::IDENTITY, z);
                current_x += size.x/2.;
            }
        }
    }
}

