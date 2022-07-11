extern crate quicksilver;
use quicksilver::{
    Result,
    geom::{Rectangle, Transform},
    graphics::{Background::Img, Background::Col, Color, Image as QSImage},
    lifecycle::{Event, Window},
    input::MouseButton,
};

use crate::lemmings::models::*;
use Scene;
use super::EventAction;
use qs_helpers::*;
use super::level_preview::LevelPreview;

// This lets you choose which level you want to play, from a given skill level.
pub struct LevelSelection {
    game: Game,
    levels: Vec<Level>,

    background: QSImage,
    frame: QSImage,
    skill: QSImage,
    font: QSMenuFont,

    mouse_was_down: bool,
    hovered_level_index: usize,
    selected_level_index: usize,
}

const MENU_TOP: f32 = 40.;
const FONT_HEIGHT: f32 = 4.;
const FRAME_OFFSET_Y: f32 = 12.5;
const FRAME_OFFSET_X: f32 = 4.5;

impl LevelSelection {
    pub fn new(game: Game, levels: Vec<Level>, background: QSImage, frame: QSImage, skill: QSImage) -> Result<LevelSelection> {
        let font = qs_font_from_lemmings_menu_font_with_scale(&game.main.main_menu.menu_font, SCALE/2)?;
        // let font = game.main.game_font.qs_font()?;
        Ok(LevelSelection {
            game,
            levels,
            background,
            frame,
            skill,
            font,
            mouse_was_down: false,
            hovered_level_index: 0,
            selected_level_index: 0,
        })
    }
}

impl Scene for LevelSelection {
    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<Vec<EventAction>> {
        let mut actions: Vec<EventAction> = Vec::new();
        match event {
            Event::MouseButton(MouseButton::Left, state) => {
                if self.mouse_was_down && !state.is_down() {
                    self.selected_level_index = self.hovered_level_index;
                    actions.push(EventAction::BeginFadeOut);
                }
                self.mouse_was_down = state.is_down();
            },
            _ => {}
        };
        Ok(actions)
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {        
        std::thread::yield_now();
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

        // Skill.
        {
            let frame_size = self.frame.area().size;
            let frame_x = (SCREEN_WIDTH/2. - frame_size.x/4.).round();
            let frame_y = 4. * SCALE as f32;
            window.draw(&Rectangle::new((frame_x, frame_y), (frame_size.x/2., frame_size.y/2.)), Img(&self.frame));
            
            let skill_size = self.skill.area().size;
            let skill_x = (SCREEN_WIDTH/2. - skill_size.x/4. + FRAME_OFFSET_X*SCALE as f32).round();
            let skill_y = frame_y + FRAME_OFFSET_Y*SCALE as f32;
            window.draw(&Rectangle::new((skill_x, skill_y), (skill_size.x/2., skill_size.y/2.)), Img(&self.skill));
        }

        // Levels.
        {
            let mouse = window.mouse();        
            let mut y: f32 = MENU_TOP * SCALE as f32;
            let row_height: f32 = if self.levels.len() > 20 { 4. } else { 6. };
            for (index, level) in self.levels.iter().enumerate() {
                let this_bottom = y + row_height * SCALE as f32;
                if y < mouse.pos().y && mouse.pos().y <= this_bottom {
                    self.hovered_level_index = index;
                    let alpha: f32 = if mouse[MouseButton::Left].is_down() { 0.2 } else { 0.1 };
                    window.draw_ex(
                        &Rectangle::new((0, y), (SCREEN_WIDTH, row_height * SCALE as f32)),
                        Col(Color { r: 1., g: 1., b: 1., a: alpha }),
                        Transform::IDENTITY,
                        1);
                }
                let draw_y = ((y + this_bottom - FONT_HEIGHT * SCALE as f32) / 2.).round();
                self.font.draw(window, 40. * SCALE as f32, draw_y, &level.name, 2.);
                y = this_bottom;
            }
        }

        Ok(())
    }

    fn next_scene(&mut self) -> Result<Option<Box<dyn Scene>>> {
        let level = self.levels[self.selected_level_index].clone();
        let scene = LevelPreview::new(self.game.clone(), self.selected_level_index, level, self.background.clone())?;
        Ok(Some(Box::new(scene)))
    }

}
