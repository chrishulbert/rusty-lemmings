use std::thread;

extern crate quicksilver;

use quicksilver::{
    Result,
    geom::{Rectangle, Vector, Transform},
    graphics::{Background::Img, Background::Col, Color, Image as QSImage, PixelFormat},
    lifecycle::{Event, Settings, State, Window, run},
    input::MouseButton,
};

use lemmings::{loader, models::*};
use super::Scene;
use crate::qs_helpers::*;

pub struct GameSelection {
    games: Games,
    background: QSImage,
    logo: QSImage,
    font: QSMenuFont,
}

const MENU_TOP: f32 = 80.;
const MENU_ROW_HEIGHT: f32 = 12.;
const MENU_FONT_WIDTH: f32 = 4.;
const MENU_FONT_HEIGHT: f32 = 8.;

impl GameSelection {
    pub fn new() -> Result<GameSelection> {
        let games = loader::load()?;
        let background = qs_image_from_lemmings_image(&games.lemmings.as_ref().unwrap().main.main_menu.background)?;
        let logo = qs_image_from_lemmings_image(&games.lemmings.as_ref().unwrap().main.main_menu.logo)?;
        let font = qs_font_from_lemmings_menu_font(&games.lemmings.as_ref().unwrap().main.main_menu.menu_font)?;
        Ok(GameSelection {
            games,
            background,
            logo,
            font,
        })
    }
}

impl Scene for GameSelection {
    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<()> {
        // match event {
        //     Event::MouseMoved(vector) => {
        //         let scaled_y = (vector.y / SCALE as f32).round() as i32;
        //         self.current_menu_hover_index = (scaled_y - MENU_TOP as i32) / MENU_ROW_HEIGHT as i32;
        //         // if vector.overlaps_rectangle(&self.crosshair_rect) {
        //         //     window.set_cursor(MouseCursor::Crosshair); // use qs input::MouseCursor
        //         // } else if vector.overlaps_rectangle(&self.grab_rect) {
        //         //     window.set_cursor(MouseCursor::Grab);
        //         // } else {
        //         //     window.set_cursor(MouseCursor::Default);
        //         // }
        //     }
        //     _ => {}
        // };
        Ok(())
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {        
        thread::yield_now();
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        // Background.
        {
            let size = self.background.area().size;
            let tiles_x: i32 = (SCREEN_WIDTH  / size.x * 2.).ceil() as i32;
            let tiles_y: i32 = (SCREEN_HEIGHT / size.y * 2.).ceil() as i32;
            for x in 0..tiles_x {
                for y in 0..tiles_y {
                    let is_even = y & 1 == 0;
                    if is_even {
                        window.draw(&Rectangle::new((x * (size.x as i32 / 2), y * (size.y as i32 / 2)), (size.x/2., size.y/2.)), Img(&self.background));
                    } else {
                        // Background image tiles badly as-is, so mirror it for a slightly better effect.
                        window.draw(&Rectangle::new((x * (size.x as i32 / 2), (y+1) * (size.y as i32 / 2)), (size.x/2., -size.y/2.)), Img(&self.background));
                    }
                }
            }
        }

        // Logo.
        {
            // Menu was 640x480 instead of 320x200, so things are half-sized. Helpfully for us that gives us retina for free.
            let size = self.logo.area().size;
            let x = (SCREEN_WIDTH - size.x/2.).round();
            window.draw(&Rectangle::new((x, 4*SCALE), (size.x/2., size.y/2.)), Img(&self.logo));
            // The scaling algo means that if we overlay the eyes with the blinking, it looks awful, so lets not do that.
        }

        // Game list.
        {
            let mouse = window.mouse();        
            let mut y: f32 = MENU_TOP * SCALE as f32;
            for game in self.games.as_vec() {
                let this_bottom = y + MENU_ROW_HEIGHT * SCALE as f32;
                if y < mouse.pos().y && mouse.pos().y <= this_bottom {
                    let alpha: f32 = if mouse[MouseButton::Left].is_down() { 0.2 } else { 0.1 };
                    window.draw_ex(
                        &Rectangle::new((0, y), (SCREEN_WIDTH, MENU_ROW_HEIGHT * SCALE as f32)),
                        Col(Color { r: 1., g: 1., b: 1., a: alpha }),
                        Transform::IDENTITY,
                        1);
                }
                let draw_y = ((y + this_bottom - MENU_FONT_HEIGHT * SCALE as f32) / 2.).round();
                self.font.draw(window, 40. * SCALE as f32, draw_y, &game.name, 2.);
                y = this_bottom;
            }
        }

            window.draw(&Rectangle::new((0, 0), (20, 20)), Col(Color::BLUE));

        Ok(())
    }
}
