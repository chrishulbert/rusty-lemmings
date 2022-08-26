use bevy::prelude::*;
use bevy::app::AppExit;
use crate::GameTextures;
use crate::GameState;
use crate::level_selection_menu::MainMenuSkillSelection;
use crate::{POINT_SIZE, TEXTURE_SCALE, FRAME_DURATION};
use crate::menu_common::{spawn_menu_background, button_highlight_system};
use crate::fadeout::create_fadeout;

#[derive(Component)]
struct MainMenuComponent; // Marker component so the menu can be despawned.

#[derive(Component)]
struct BlinkAnimationTimer {
    timer: Timer,
    index: isize, // Set to negative if you want a delay before the blink.
    dwell: isize, // Dwell time in frames after a blink.
}

fn animate_blinking_sprites(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut BlinkAnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in &mut query {
        timer.timer.tick(time.delta());
        if timer.timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            let texture_frames = texture_atlas.textures.len() as isize;

            // Advance.
            timer.index += 1;
            let total_frames = texture_frames + timer.dwell;
            if timer.index > total_frames {
                timer.index = 0;
            }

            // Apply it. If it's in the initial delay or the dwell time, just show 0.
            if 0 <= timer.index && timer.index < texture_frames {
                sprite.index = timer.index as usize;
            } else {
                sprite.index = 0;
            }
        }
    }
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::MainMenu)
                // .with_system(enter)
                .with_system(spawn_menu_logo)
                .with_system(spawn_menu_buttons)                
                .with_system(spawn_background)
        )
        .add_system_set(
            SystemSet::on_update(GameState::MainMenu)
                .with_system(button_system)
                .with_system(button_highlight_system)
                .with_system(animate_blinking_sprites)
                .with_system(hover_highlight_system)
        )
        .add_system_set(
            SystemSet::on_exit(GameState::MainMenu)
                .with_system(exit)
        );
	}
}

pub enum MainMenuButtonAction {
    Skill(isize),
    Settings,
    Exit,
}

#[derive(Component)]
pub struct MainMenuButton{
    pub action: MainMenuButtonAction,
}

fn hover_highlight_system(
    windows: Res<Windows>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut buttons: Query<(&mut Sprite, &Transform), (With<MainMenuButton>)>,
) {
    if let Some(window) = windows.iter().next() {
        let position = window.cursor_position().unwrap_or(Vec2::NEG_ONE);
        let x = position.x - window.width() / 2.;
        let y = position.y - window.height() / 2.;
        for (mut sprite, transform) in &mut buttons {
            let is_over = 
                transform.translation.x - 120. <= x && x <= transform.translation.x + 120. &&
                transform.translation.y - 61. <= y && y <= transform.translation.y + 61.;
            let a: f32 = if is_over { if mouse_buttons.pressed(MouseButton::Left) { 0.5 } else { 0.8 } } else { 1. };
            sprite.color.set_a(a);
        }
    }
}

fn button_system(
    windows: Res<Windows>,
    mouse_buttons: Res<Input<MouseButton>>,
    buttons: Query<(&Transform, &MainMenuButton)>,
    game_textures: Res<GameTextures>,
    mut skill: ResMut<MainMenuSkillSelection>,
    mut commands: Commands,
    mut exit: EventWriter<AppExit>,
) {
    if mouse_buttons.just_released(MouseButton::Left) {
        if let Some(window) = windows.iter().next() {
            if let Some(position) = window.cursor_position() {
                let x = position.x - window.width() / 2.;
                let y = position.y - window.height() / 2.;
                let button_o = buttons.iter().find(|&b| {
                    b.0.translation.x - 120. <= x && x <= b.0.translation.x + 120. &&
                    b.0.translation.y - 61. <= y && y <= b.0.translation.y + 61.
                });
                if let Some(button) = button_o {
                    let mmb: &MainMenuButton = button.1;
                    match mmb.action {
                        MainMenuButtonAction::Skill(skill_level) => {
                            skill.0 = skill_level;
                            create_fadeout(&mut commands, GameState::LevelSelectionMenu, &game_textures);
                        },
                        MainMenuButtonAction::Settings => {
                            println!("Settings TODO");
                        },
                        MainMenuButtonAction::Exit => {
                            exit.send(AppExit);
                        },
                    }
                }
            }
        }    
    }
}

fn exit(
    mut commands: Commands,
    menu_components: Query<Entity, With<MainMenuComponent>>,
) {
    for e in menu_components.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn spawn_background(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
) {
    commands
        .spawn_bundle(SpatialBundle::default())
        .insert(MainMenuComponent)
        .with_children(|parent| {
            spawn_menu_background(parent, &game_textures);
        });
}

fn spawn_menu_logo(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
) {
    const SCALE: f32 = TEXTURE_SCALE / 2.; // Logo is SVGA so halve it.
    fn spawn_blink(parent: &mut ChildBuilder, atlas: Handle<TextureAtlas>, x: f32, y: f32, index: isize, dwell: isize) {
        parent.spawn_bundle(SpriteSheetBundle {
            texture_atlas: atlas,
            transform: Transform{
                translation: Vec3::new(x * POINT_SIZE / SCALE, y * POINT_SIZE / SCALE, 2.),
                ..default()
            },        
            ..default()
        }).insert(BlinkAnimationTimer{
            timer: Timer::from_seconds(FRAME_DURATION, true),
            index,
            dwell,
        });
    }

    commands
        .spawn_bundle(SpriteBundle {
            texture: game_textures.logo.clone(),
            transform: Transform{
                translation: Vec3::new(0., 52. * POINT_SIZE, 1.), // 52 -> make it overlap the background-tile-seam.
                scale: Vec3::new(SCALE, SCALE, 1.),
                ..default()
            },        
            ..default()
        })
        .insert(MainMenuComponent)
        .with_children(|parent| {
            spawn_blink(parent, game_textures.blink1.clone(), -138., 1., -15, 30);
            spawn_blink(parent, game_textures.blink2.clone(), -26., 2., -22, 45);
            spawn_blink(parent, game_textures.blink3.clone(), 94., 5.5, -30, 37);
        });
}

fn spawn_menu_buttons(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
) {
    fn spawn_button(commands: &mut ChildBuilder, sign: Handle<Image>, skill: Option<Handle<Image>>, x: f32, y: f32, action: MainMenuButtonAction) {
        commands.spawn_bundle(SpriteBundle{
            texture: sign,
            transform: Transform{
                translation: Vec3::new(x * POINT_SIZE, y * POINT_SIZE, 2.),
                scale: Vec3::new(TEXTURE_SCALE / 2., TEXTURE_SCALE / 2., 1.), // Menu is svga, so halve everything.
                ..default()
            },
            ..default()
        }).insert(MainMenuButton{
            action,
        }).with_children(|parent| {
            if let Some(skill) = skill {
                parent.spawn_bundle(SpriteBundle{
                    texture: skill,
                    transform: Transform{
                        translation: Vec3::new(0., -28. * POINT_SIZE, 3.),
                        ..default()
                    },        
                    ..default()
                });
                }
        });
    }

    commands
        .spawn_bundle(SpatialBundle {
            ..default()
        })
        .insert(MainMenuComponent)
        .with_children(|parent| {
            spawn_button(parent, game_textures.f1.clone(), Some(game_textures.fun.clone()), -100., 0., MainMenuButtonAction::Skill(0));
            spawn_button(parent, game_textures.f2.clone(), Some(game_textures.tricky.clone()), -33., 0., MainMenuButtonAction::Skill(1));
            spawn_button(parent, game_textures.f3.clone(), Some(game_textures.taxing.clone()), 33., 0., MainMenuButtonAction::Skill(2));
            spawn_button(parent, game_textures.level_rating.clone(), Some(game_textures.mayhem.clone()), 100., 0., MainMenuButtonAction::Skill(3));
            spawn_button(parent, game_textures.exit_to_dos.clone(), None, -33., -40., MainMenuButtonAction::Exit);
            spawn_button(parent, game_textures.f4_settings.clone(), None, 33., -40., MainMenuButtonAction::Settings);
        });
}
