use std::thread;
use std::boxed::Box;
use std::io::{Error, ErrorKind};

extern crate quicksilver;
use quicksilver::{
    Result,
    geom::{Rectangle, Vector, Transform, Shape},
    graphics::{Background::Img, Background::Col, Color, Image as QSImage, PixelFormat},
    lifecycle::{Event, Settings, State, Window, run},
    input::MouseButton,
};

use lemmings::models::*;
use lemmings::levels_per_game_and_skill;
use Scene;
use super::EventAction;
use qs_helpers::*;
use super::level_selection::LevelSelection;

// This is the screen that allows you to choose which skill level you'd like to play at.
pub struct SkillSelection {
    game: Game,
    background: QSImage,
    logo: QSImage,
    frame: QSImage,
    skills: Vec<QSImage>,
    mouse_was_down: bool,
    hovered_skill: isize,
    selected_skill: isize,
}

const SKILLS_TOP: f32 = 80.;
const SKILL_WIDTH: f32 = 64.;
const FRAME_OFFSET_Y: f32 = 12.5;
const FRAME_OFFSET_X: f32 = 4.5;
const BUTTON_MARGIN_Y: f32 = 4.;

impl SkillSelection {
    pub fn new(game: Game, background: QSImage) -> Result<SkillSelection> {
        let logo = qs_image_from_lemmings_image(&game.main.main_menu.logo)?;
        let frame = qs_image_from_lemmings_image(&game.main.main_menu.level_rating)?;
        let skill_0 = qs_image_from_lemmings_image(&game.main.main_menu.fun)?;
        let skill_1 = qs_image_from_lemmings_image(&game.main.main_menu.tricky)?;
        let skill_2 = qs_image_from_lemmings_image(&game.main.main_menu.taxing)?;
        let skill_3 = qs_image_from_lemmings_image(&game.main.main_menu.mayhem)?;
        // TODO extra skill for oh no more lemmings
        Ok(SkillSelection {
            game,
            background,
            logo,
            frame,
            skills: vec![skill_0, skill_1, skill_2, skill_3],
            mouse_was_down: false,
            hovered_skill: 0,
            selected_skill: 0,
        })
    }
}

impl Scene for SkillSelection {
    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<Vec<EventAction>> {
        let mut actions: Vec<EventAction> = Vec::new();
        match event {
            Event::MouseButton(MouseButton::Left, state) => {
                if self.mouse_was_down && !state.is_down() {
                    self.selected_skill = self.hovered_skill;
                    actions.push(EventAction::BeginFadeOut);
                }
                self.mouse_was_down = state.is_down();
            },
            _ => {}
        };
        Ok(actions)
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
            let mut x: f32 = (SCREEN_WIDTH / 2. - self.skills.len() as f32 / 2. * SKILL_WIDTH * SCALE as f32).round();
            let y: f32 = SKILLS_TOP * SCALE as f32;
            let skill_y: f32 = y + FRAME_OFFSET_Y * SCALE as f32;
            let mouse = window.mouse();
            let frame_size = self.frame.area().size;
            let button_top = y - BUTTON_MARGIN_Y * SCALE as f32;
            let button_height = frame_size.y / 2. + 2. * BUTTON_MARGIN_Y * SCALE as f32;
            for (index, skill) in self.skills.iter().enumerate() {
                let this_right = x + SKILL_WIDTH * SCALE as f32;
                let this_mid_x = x + SKILL_WIDTH * SCALE as f32 / 2.;

                let button_area = Rectangle::new((x, button_top), (this_right - x, button_height));
                if button_area.contains(mouse.pos()) {
                    self.hovered_skill = index as isize;
                    let alpha: f32 = if mouse[MouseButton::Left].is_down() { 0.2 } else { 0.1 };
                    window.draw_ex(
                        &button_area,
                        Col(Color { r: 1., g: 1., b: 1., a: alpha }),
                        Transform::IDENTITY,
                        1);
                }
                {
                    let draw_x = (this_mid_x - frame_size.x / 4.).round();
                    window.draw_ex(&Rectangle::new((draw_x, y), (frame_size.x/2., frame_size.y/2.)), Img(&self.frame), Transform::IDENTITY, 2);
                }
                {
                    let size = skill.area().size;
                    let draw_x = (this_mid_x - size.x / 4.).round();
                    window.draw_ex(&Rectangle::new((draw_x + FRAME_OFFSET_X * SCALE as f32, skill_y), (size.x/2., size.y/2.)), Img(skill), Transform::IDENTITY, 3);
                }

                // let draw_y = ((y + this_bottom - MENU_FONT_HEIGHT * SCALE as f32) / 2.).round();
                // self.font.draw(window, 40. * SCALE as f32, draw_y, &game.name, 2.);
                x = this_right;
            }
        }

        Ok(())
    }

    fn next_scene(&mut self) -> Result<Option<Box<dyn Scene>>> {
        let levels = levels_per_game_and_skill::levels_per_game_and_skill(&self.game.id, self.selected_skill, &self.game.levels);
        let scene = LevelSelection::new(self.game.clone(), 
            levels,
            self.background.clone(),
            self.frame.clone(),
            self.skills[self.selected_skill as usize].clone())?;
        Ok(Some(Box::new(scene)))
    }

}
