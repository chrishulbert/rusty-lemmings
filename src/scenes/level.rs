// This is the scene for playing an actual level.

extern crate quicksilver;
use quicksilver::{
    Result,
    geom::{Rectangle, Transform, Vector},
    graphics::{Background::Img, Background::Col, Color, Image as QSImage},
    lifecycle::{Event, Window},
    input::MouseButton,
};

use std::collections::HashMap;
use lemmings::models::*;
use Scene;
use EventAction;
use qs_helpers::*;
use lemmings::level_renderer::{self, RenderedLevel};
use xbrz;

const SKILL_PANEL_SCALE: u8 = 5;
const SKILL_WIDTH: f32 = 16.;
const SKILL_HEIGHT: f32 = 24.;
const SKILL_PANEL_GRAPHIC_HEIGHT: f32 = 40.;
const SKILL_BUTTONS: usize = 12;
const AUTO_SCROLL_SLOW_MARGIN: f32 = 30.;
const AUTO_SCROLL_FAST_MARGIN: f32 = 10.;
const GAME_AREA_HEIGHT: f32 = 960.;

type SliceMap = HashMap<isize, QSImage>;

// const STRIP_WIDTH_GAME_PX: usize = 11; // This way each vertical strip of the map is just < 1MB. Eg 160 high * 12 scale * 4 rgba * 11 game pixels wide * 12 scale = 0.97MB

pub struct LevelScene {
    game: Game,
    level_index: usize,
    level: Level,
    font: QSGameFont,
    skill_panel: QSImage,
    mouse_was_down: bool,
    level_unscaled: RenderedLevel,
    level_scaled_image: Image,
    level_slices: SliceMap,
    scroll_offset_x: isize, // In game pixels.
    selected_skill_index: isize,
}

// Skill panel only gets buttons displayed. At 5x, thus covers 5*24=120pt. Level is 160x6=960. 120+960 = 1080 full.
// Only regret is that the buttons don't fill the whole width, but it looks good left-aligned with text to its right.
// Skill panel minimap has 2-game-px margin on top and bottom, thus 20-game-pt high (1/8th original game px). Aka 20*5=100pt / 200px high.
// Map is 1040px wide, aka 520pt wide, aka 104-game-px wide. Thus supporting a max map width of 832px.
impl LevelScene {
    pub fn new(game: Game, level_index: usize, level: Level) -> Result<LevelScene> {
        let font = game.main.game_font.qs_font()?;
        let skill_panel = qs_image_from_lemmings_image_scaled_twice(&game.main.skill_panel, SKILL_PANEL_SCALE, 2)?;
        println!("Rendering level");
        let render = level_renderer::render(&level, &game.grounds, &game.specials)?; // This renders in game pixels.
        println!("Scaling level A");
        let scaled_half = xbrz::scale(SCALE, &render.image.bitmap, render.image.width as u32, render.image.height as u32); // Scale the whole thing in one go so there are no seams.
        println!("Scaling level B");
        let scaled = xbrz::scale(2, &scaled_half, render.image.width as u32 * SCALE as u32, render.image.height as u32 * SCALE as u32);
        let scaled_width: usize = render.image.width * SCALE as usize * 2;
        let scaled_height: usize = render.image.height * SCALE as usize * 2;
        println!("Slicing level");
        // Slice it into game pixels.
        let mut slices = SliceMap::new();
        for image_x in 0..render.image.width {
            let effective_scale: usize = SCALE as usize * 2;
            let pixels_capacity: usize = effective_scale * scaled_height as usize;
            let mut slice: Vec<u32> = Vec::with_capacity(pixels_capacity);
            let mut source_offset: usize = image_x * effective_scale;
            let stride: usize = scaled_width - effective_scale;
            for _pixel_y in 0..scaled_height { // With any luck, these nested loops will be vectorised by the compiler into something fast.
                for _pixel_x in 0..effective_scale {
                    slice.push(scaled[source_offset]);
                    source_offset += 1;
                }
                source_offset += stride;
            }
            let slice_image = Image {
                bitmap: slice,
                width: effective_scale,
                height: scaled_height,
            };
            let slice_qs_image = qs_image_from_lemmings_image_no_scale(&slice_image)?;
            let game_x: isize = image_x as isize + render.size.min_x;
            slices.insert(game_x, slice_qs_image);
        }
        println!("Finished slicing");

        let level_scaled_image = Image {
            bitmap: scaled,
            width: scaled_width,
            height: scaled_height,
        };
        let scroll_offset_x = level.globals.start_screen_xpos as isize;
        Ok(LevelScene {
            game,
            level_index,
            level,
            font,
            skill_panel,
            mouse_was_down: false,
            level_unscaled: render,
            level_scaled_image,
            level_slices: slices,
            scroll_offset_x,
            selected_skill_index: 0,
        })
    }

    fn on_mouse_down_in_game_area(&mut self, mouse: &Vector) {

    }

    fn on_mouse_down_in_skill_bar(&mut self, mouse: &Vector) {
        let index = mouse.x as isize / (SKILL_WIDTH as isize * SKILL_PANEL_SCALE as isize);
        if index < SKILL_BUTTONS as isize {
            match index {
                // 0 => { - },
                // 1 => { + },
                2 => { self.selected_skill_index = index - 2 }, // climb
                3 => { self.selected_skill_index = index - 2 }, // umbrella
                4 => { self.selected_skill_index = index - 2 }, // explode
                5 => { self.selected_skill_index = index - 2 }, // block
                6 => { self.selected_skill_index = index - 2 }, // build
                7 => { self.selected_skill_index = index - 2 }, // bash
                8 => { self.selected_skill_index = index - 2 }, // diagonal dig
                9 => { self.selected_skill_index = index - 2 }, // vertical dig
                // 10 >= {  pause },
                // 11 >= {  explode all },
                _ => {},
            }
        }
    }
}

impl Scene for LevelScene {
    fn event(&mut self, event: &Event, window: &mut Window) -> Result<Vec<EventAction>> {
        let mouse_pos = window.mouse().pos();
        let mut actions: Vec<EventAction> = Vec::new();
        match event {
            Event::MouseButton(MouseButton::Left, state) => {
                if !self.mouse_was_down && state.is_down() { // Start a click.
                    if mouse_pos.y > GAME_AREA_HEIGHT {
                        self.on_mouse_down_in_skill_bar(&mouse_pos)
                    } else {
                        self.on_mouse_down_in_game_area(&mouse_pos)
                    }
                }
                self.mouse_was_down = state.is_down();
            },
            _ => {}
        };
        Ok(actions)
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {        
        let mouse_pos = window.mouse().pos();

        if mouse_pos.y < GAME_AREA_HEIGHT {
            if mouse_pos.x < AUTO_SCROLL_FAST_MARGIN * SCALE as f32 {
                self.scroll_offset_x -= 2;
            } else if mouse_pos.x > SCREEN_WIDTH - AUTO_SCROLL_FAST_MARGIN * SCALE as f32 {
                self.scroll_offset_x += 2;
            } else if mouse_pos.x < AUTO_SCROLL_SLOW_MARGIN * SCALE as f32 {
                self.scroll_offset_x -= 1;
            } else if mouse_pos.x > SCREEN_WIDTH - AUTO_SCROLL_SLOW_MARGIN * SCALE as f32 {
                self.scroll_offset_x += 1;
            }

            // Clamp it to half a screen beyond the edges of the map.
            let half: isize = SCREEN_WIDTH as isize / SCALE as isize / 2;
            let min_x = self.level_unscaled.size.min_x - half;
            let max_x = self.level_unscaled.size.max_x - half;
            if self.scroll_offset_x < min_x {
                self.scroll_offset_x = min_x;
            } else if self.scroll_offset_x > max_x {
                self.scroll_offset_x = max_x;
            }
        }

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
                let alpha: f32 = if mouse[MouseButton::Left].is_down() { 0.4 } else { 0.2 };
                let x: f32 = ((index as f32 * SKILL_WIDTH) + 1.) * SKILL_PANEL_SCALE as f32;
                window.draw_ex(
                    &Rectangle::new((x, skill_top), ((SKILL_WIDTH - 1.) * SKILL_PANEL_SCALE as f32, (SKILL_HEIGHT - 1.) * SKILL_PANEL_SCALE as f32)),
                    Col(Color { r: 1., g: 1., b: 1., a: alpha }),
                    Transform::IDENTITY,
                    3);
            }

            // Skill selection box.
            window.draw_ex( // Left side of box.
                &Rectangle::new(((self.selected_skill_index + 2) as f32 * SKILL_WIDTH * SKILL_PANEL_SCALE as f32, GAME_AREA_HEIGHT),
                    (SKILL_PANEL_SCALE as f32, SKILL_HEIGHT * SKILL_PANEL_SCALE as f32)),
                Col(Color { r: 1., g: 1., b: 1., a: 1. }),
                Transform::IDENTITY,
                4);
            window.draw_ex( // Right side.
                &Rectangle::new(((self.selected_skill_index + 3) as f32 * SKILL_WIDTH * SKILL_PANEL_SCALE as f32, GAME_AREA_HEIGHT),
                    (SKILL_PANEL_SCALE as f32, SKILL_HEIGHT * SKILL_PANEL_SCALE as f32)),
                Col(Color { r: 1., g: 1., b: 1., a: 1. }),
                Transform::IDENTITY,
                4);
            window.draw_ex( // Top.
                &Rectangle::new(((self.selected_skill_index + 2) as f32 * SKILL_WIDTH * SKILL_PANEL_SCALE as f32, GAME_AREA_HEIGHT),
                    (SKILL_WIDTH * SKILL_PANEL_SCALE as f32, SKILL_PANEL_SCALE as f32)),
                Col(Color { r: 1., g: 1., b: 1., a: 1. }),
                Transform::IDENTITY,
                4);
            window.draw_ex( // Bottom.
                &Rectangle::new(((self.selected_skill_index + 2) as f32 * SKILL_WIDTH * SKILL_PANEL_SCALE as f32, GAME_AREA_HEIGHT + (SKILL_HEIGHT - 1.) * SKILL_PANEL_SCALE as f32),
                    (SKILL_WIDTH * SKILL_PANEL_SCALE as f32, SKILL_PANEL_SCALE as f32)),
                Col(Color { r: 1., g: 1., b: 1., a: 1. }),
                Transform::IDENTITY,
                4);
        }

        // Level.
        {
            // Each slice is 1 game pixel, which is SCALE points, or SCALE*2 retina px.
            let displayed_slices: isize = SCREEN_WIDTH as isize / SCALE as isize;
            for screen_slice in 0..displayed_slices {
                let game_slice = self.scroll_offset_x + screen_slice;
                if let Some(slice) = self.level_slices.get(&game_slice) {                    
                    window.draw_ex(&Rectangle::new((screen_slice as f32 * SCALE as f32, 0), (slice.area().size.x / 2., slice.area().size.y / 2.)), Img(slice), Transform::IDENTITY, 3);
                }
            }
            // let size = self.level_image.area().size;
            // let w: f32 = size.x / 2.;
            // let h: f32 = size.y / 2.;
            // let x: f32 = ((SCREEN_HEIGHT - w)/2.).round();
            // let y: f32 = 0.;
            // window.draw_ex(&Rectangle::new((x, y), (w, h)), Img(&self.level_image), Transform::IDENTITY, 1);
        }

        // Scroll highlights.
        {
            if mouse_pos.y < GAME_AREA_HEIGHT {
                if mouse_pos.x < AUTO_SCROLL_FAST_MARGIN * SCALE as f32 {
                    window.draw_ex(
                        &Rectangle::new((0, 0), (AUTO_SCROLL_FAST_MARGIN * SCALE as f32, GAME_AREA_HEIGHT)),
                        Col(Color { r: 1., g: 1., b: 1., a: 0.2 }),
                        Transform::IDENTITY,
                        4);
                } else if mouse_pos.x > SCREEN_WIDTH - AUTO_SCROLL_FAST_MARGIN * SCALE as f32 {
                    window.draw_ex(
                        &Rectangle::new((SCREEN_WIDTH - AUTO_SCROLL_FAST_MARGIN * SCALE as f32, 0), (AUTO_SCROLL_FAST_MARGIN * SCALE as f32, GAME_AREA_HEIGHT)),
                        Col(Color { r: 1., g: 1., b: 1., a: 0.2 }),
                        Transform::IDENTITY,
                        4);
                } else if mouse_pos.x < AUTO_SCROLL_SLOW_MARGIN * SCALE as f32 {
                    window.draw_ex(
                        &Rectangle::new((0, 0), (AUTO_SCROLL_SLOW_MARGIN * SCALE as f32, GAME_AREA_HEIGHT)),
                        Col(Color { r: 1., g: 1., b: 1., a: 0.1 }),
                        Transform::IDENTITY,
                        4);
                } else if mouse_pos.x > SCREEN_WIDTH - AUTO_SCROLL_SLOW_MARGIN * SCALE as f32 {
                    window.draw_ex(
                        &Rectangle::new((SCREEN_WIDTH - AUTO_SCROLL_SLOW_MARGIN * SCALE as f32, 0), (AUTO_SCROLL_SLOW_MARGIN * SCALE as f32, GAME_AREA_HEIGHT)),
                        Col(Color { r: 1., g: 1., b: 1., a: 0.1 }),
                        Transform::IDENTITY,
                        4);
                }
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
