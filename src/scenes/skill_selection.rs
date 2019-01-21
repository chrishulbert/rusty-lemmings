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
use Scene;
use qs_helpers::*;

// This is the screen that allows you to choose which skill level you'd like to play at.
pub struct SkillSelection {
    game: Game,
    background: QSImage,
    logo: QSImage,
    frames_skills: Vec<(QSImage, QSImage)>,
}

const SKILLS_TOP: f32 = 100.;
const SKILL_WIDTH: f32 = 64.;
const FRAME_OFFSET: f32 = 4.;

impl SkillSelection {
    pub fn new(game: Game, background: QSImage) -> Result<SkillSelection> {
        let logo = qs_image_from_lemmings_image(&game.main.main_menu.logo)?;
        let frame_0 = qs_image_from_lemmings_image(&game.main.main_menu.f1)?;
        let frame_1 = qs_image_from_lemmings_image(&game.main.main_menu.f2)?;
        let frame_2 = qs_image_from_lemmings_image(&game.main.main_menu.f3)?;
        let frame_3 = qs_image_from_lemmings_image(&game.main.main_menu.level_rating)?;
        let skill_0 = qs_image_from_lemmings_image(&game.main.main_menu.fun)?;
        let skill_1 = qs_image_from_lemmings_image(&game.main.main_menu.tricky)?;
        let skill_2 = qs_image_from_lemmings_image(&game.main.main_menu.taxing)?;
        let skill_3 = qs_image_from_lemmings_image(&game.main.main_menu.mayhem)?;
        Ok(SkillSelection {
            game,
            background,
            logo,
            frames_skills: vec![(frame_0, skill_0), (frame_1, skill_1), (frame_2, skill_2), (frame_3, skill_3)],
        })
    }
}

impl Scene for SkillSelection {
    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<()> {
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

        // Skill list.
        {
            let mut x: f32 = (SCREEN_WIDTH / 2. - self.frames_skills.len() as f32 / 2. * SKILL_WIDTH).round();
            let mouse = window.mouse();
            for (frame, skill) in self.frames_skills.iter() {
                let this_right = x + SKILL_WIDTH * SCALE as f32;
                let this_mid_x = x + SKILL_WIDTH * SCALE as f32 / 2.;
                // BUTTON_AREA.contains(window.mouse().pos())
                // mouse.pos().
                // if y < mouse.pos().y && mouse.pos().y <= this_bottom {
                //     let alpha: f32 = if mouse[MouseButton::Left].is_down() { 0.2 } else { 0.1 };
                //     window.draw_ex(
                //         &Rectangle::new((0, y), (SCREEN_WIDTH, MENU_ROW_HEIGHT * SCALE as f32)),
                //         Col(Color { r: 1., g: 1., b: 1., a: alpha }),
                //         Transform::IDENTITY,
                //         1);
                // }
                {
                    let size = frame.area().size;
                    let draw_x = (this_mid_x - size.x / 2.).round();
                    window.draw(&Rectangle::new((x, SKILLS_TOP), (size.x/2., size.y/2.)), Img(frame));
                }
                {
                    let size = skill.area().size;
                    let draw_x = (this_mid_x - size.x / 2.).round();
                    window.draw(&Rectangle::new((x, SKILLS_TOP + FRAME_OFFSET * SCALE as f32), (size.x/2., size.y/2.)), Img(skill));
                }

                // let draw_y = ((y + this_bottom - MENU_FONT_HEIGHT * SCALE as f32) / 2.).round();
                // self.font.draw(window, 40. * SCALE as f32, draw_y, &game.name, 2.);
                x = this_right;
            }
        }

        Ok(())
    }
}
