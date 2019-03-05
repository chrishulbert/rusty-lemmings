use std::format;

extern crate quicksilver;
use quicksilver::{
    Result,
    geom::{Rectangle, Transform},
    graphics::{Background::Img, Background::Col, Color, Image as QSImage},
    lifecycle::{Event, Window},
    input::MouseButton,
};

use lemmings::models::*;
use Scene;
use super::EventAction;
use qs_helpers::*;
use lemmings::level_renderer;
use scenes::level::LevelScene;

// This previews a level before you play it.
pub struct LevelPreview {
    game: Game,
    level_index: usize,
    level: Level,
    preview: QSImage,
    background: QSImage,
    font: QSMenuFont,
}

const ROW_HEIGHT: f32 = 12.;
const PREVIEW_SCALE: f32 = 4.;

impl LevelPreview {
    pub fn new(game: Game, level_index: usize, level: Level, background: QSImage) -> Result<LevelPreview> {
        let font = qs_font_from_lemmings_menu_font(&game.main.main_menu.menu_font)?;
        let render = level_renderer::render(&level, &game.grounds, &game.specials)?;
        let preview = qs_image_from_lemmings_image(&render.image)?;
        Ok(LevelPreview {
            game,
            level_index,
            level,
            preview,
            background,
            font,
        })
    }
}

impl Scene for LevelPreview {
    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<Vec<EventAction>> {
        let mut actions: Vec<EventAction> = Vec::new();
        match event {
            Event::MouseButton(MouseButton::Left, state) => {
                if state.is_down() {
                    actions.push(EventAction::BeginFadeOut);
                }
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

        // Preview.
        let preview_height: f32 = (SCREEN_HEIGHT / 3.).round();
        {
            window.draw_ex(
                &Rectangle::new((0, 0), (SCREEN_WIDTH, preview_height)),
                Col(Color { r: 0., g: 0., b: 0., a: 1. }),
                Transform::IDENTITY,
                1);

            let size = self.preview.area().size;
            let w: f32 = size.x / PREVIEW_SCALE;
            let h: f32 = size.y / PREVIEW_SCALE;
            let x: f32 = ((SCREEN_WIDTH - w)/2.).round();
            let y: f32 = ((preview_height - h)/2.).round();
            window.draw_ex(&Rectangle::new((x, y), (w, h)), Img(&self.preview), Transform::IDENTITY, 2);
        }

        // Text.
        {
            let x: f32 = (SCALE as f32 * 4.).round();
            let text = format!("Level {}", self.level_index + 1);
            self.font.draw(window, x, x, &text, 3.);
        }
        {
            let width = self.font.width(&self.level.name);
            let x: f32 = ((SCREEN_WIDTH - width) / 2.).round();
            let mut y: f32 = preview_height + ROW_HEIGHT * SCALE as f32;
            self.font.draw(window, x, y, &self.level.name, 2.);

            let x: f32 = (SCREEN_WIDTH / 4.).round();
            y += 2. * ROW_HEIGHT * SCALE as f32;
            let text = format!("Number of Lemmings: {}", self.level.globals.num_of_lemmings);
            self.font.draw(window, x, y, &text, 2.);

            y += ROW_HEIGHT * SCALE as f32;
            let text = format!("{} to be saved", self.level.globals.num_to_rescue);
            self.font.draw(window, x, y, &text, 2.);

            y += ROW_HEIGHT * SCALE as f32;
            let text = format!("Release rate: {}", self.level.globals.release_rate);
            self.font.draw(window, x, y, &text, 2.);

            y += ROW_HEIGHT * SCALE as f32;
            let text = format!("Time: {} minutes", self.level.globals.time_limit);
            self.font.draw(window, x, y, &text, 2.);

            y += 2. * ROW_HEIGHT * SCALE as f32;
            let text = "Press mouse button to continue";
            let width = self.font.width(&text);
            let x: f32 = ((SCREEN_WIDTH - width) / 2.).round();
            self.font.draw(window, x, y, &text, 2.);
        }

        Ok(())
    }

    fn next_scene(&mut self) -> Result<Option<Box<dyn Scene>>> {
        let scene = LevelScene::new(self.game.clone(), self.level_index, self.level.clone())?;
        Ok(Some(Box::new(scene)))
    }

}
