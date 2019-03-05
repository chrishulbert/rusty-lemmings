// This is the scene for playing an actual level.

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
use EventAction;
use qs_helpers::*;

const SKILL_PANEL_SCALE: u8 = 5;
const SKILL_WIDTH: f32 = 16.;
const SKILL_HEIGHT: f32 = 24.;
const SKILL_PANEL_GRAPHIC_HEIGHT: f32 = 40.;
const SKILL_BUTTONS: usize = 12;

pub struct LevelScene {
    game: Game,
    level_index: usize,
    level: Level,
    font: QSGameFont,
    skill_panel: QSImage,
    mouse_was_down: bool,
}

// Skill panel only gets buttons displayed. At 5x, thus covers 5*24=120pt. Level is 160x6=960. 120+960 = 1080 full.
// Only regret is that the buttons don't fill the whole width.

impl LevelScene {
    pub fn new(game: Game, level_index: usize, level: Level) -> Result<LevelScene> {
        let font = game.main.game_font.qs_font()?;
        let skill_panel = qs_image_from_lemmings_image_scaled_twice(&game.main.skill_panel, SKILL_PANEL_SCALE, 2)?;
        Ok(LevelScene {
            game,
            level_index,
            level,
            font,
            skill_panel,
            mouse_was_down: false,
        })
    }
}

impl Scene for LevelScene {
    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<Vec<EventAction>> {
        let mut actions: Vec<EventAction> = Vec::new();
        // match event {
        //     Event::MouseButton(MouseButton::Left, state) => {
        //         if state.is_down() {
        //             actions.push(EventAction::BeginFadeOut);
        //         }
        //     },
        //     _ => {}
        // };
        Ok(actions)
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {        
        std::thread::yield_now();
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        let mouse = window.mouse();
        let mouse_pos = mouse.pos();

        window.clear(Color::BLACK)?;

        // Bottom panel.
        {
            // Skills.
            let size = self.skill_panel.area().size;
            let w: f32 = size.x / 2.;
            let h: f32 = size.y / 2.;
            let x: f32 = 0.;
            let y: f32 = (SCREEN_HEIGHT - h).round();
            window.draw_ex(&Rectangle::new((x, y), (w, h)), Img(&self.skill_panel), Transform::IDENTITY, 1);

            // Text.
            let bar_height: f32 = (h * SKILL_HEIGHT / SKILL_PANEL_GRAPHIC_HEIGHT).round();
            let text_y_mid: f32 = SCREEN_HEIGHT - (bar_height/2.).round();
            let text_y: f32 = text_y_mid - 9. * SCALE as f32;
            let text_y_2: f32 = text_y_mid + 1. * SCALE as f32;
            let text_x: f32 = w + 2. * SCALE as f32;
            let text = format!("Out {}", 1);
            self.font.draw(window, text_x, text_y, &text, 2.);

            let text = format!("In {}%", 2);
            self.font.draw(window, text_x, text_y_2, &text, 2.);

            let text = format!("{}-{:02}", 3, 4);
            let text_w = self.font.width(&text);
            let text_x = SCREEN_WIDTH - text_w - SCALE as f32;
            self.font.draw(window, text_x, text_y_2, &text, 2.);

            // Button highlights.
            let skill_top: f32 = SCREEN_HEIGHT - bar_height;
            if mouse_pos.y >= skill_top && mouse_pos.x <= SKILL_WIDTH * SKILL_PANEL_SCALE as f32 * SKILL_BUTTONS as f32 {
                let index = mouse_pos.x as isize / (SKILL_WIDTH as isize * SKILL_PANEL_SCALE as isize);
                let alpha: f32 = if mouse[MouseButton::Left].is_down() { 0.2 } else { 0.1 };
                let x: f32 = index as f32 * SKILL_WIDTH * SKILL_PANEL_SCALE as f32;
                window.draw_ex(
                    &Rectangle::new((x, skill_top), (SKILL_WIDTH * SKILL_PANEL_SCALE as f32, SKILL_HEIGHT * SKILL_PANEL_SCALE as f32)),
                    Col(Color { r: 1., g: 1., b: 1., a: alpha }),
                    Transform::IDENTITY,
                    3);
            }
        }

        Ok(())
    }

    fn next_scene(&mut self) -> Result<Option<Box<dyn Scene>>> {
        // let selected_game = self.selected_game.take(); // Transfer ownership from the ivar.
        // if let Some(selected_game) = selected_game {
        //     let skill_selection = SkillSelection::new(selected_game, self.background.clone())?;
        //     Ok(Some(Box::new(skill_selection)))
        // } else {
            Ok(None)
        // }
    }

}
