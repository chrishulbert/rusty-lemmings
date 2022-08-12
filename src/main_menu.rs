use bevy::prelude::*;
use crate::GameTextures;
use crate::GameState;
use crate::level_selection_menu::MainMenuSkillSelection;
use crate::{POINT_SIZE, TEXTURE_SCALE, FRAME_DURATION};
use crate::menu_common::{NORMAL_BUTTON, spawn_menu_background, button_highlight_system};

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
                .with_system(spawn_menu_background)
        )
        .add_system_set(
            SystemSet::on_update(GameState::MainMenu)
                .with_system(button_system)
                .with_system(button_highlight_system)
                .with_system(animate_blinking_sprites)
        )
        .add_system_set(
            SystemSet::on_exit(GameState::MainMenu)
                .with_system(exit)
        );
	}
}

#[derive(Component)]
pub struct MainMenuSkillButton{
    pub is_clicked: bool,
    pub skill: isize,
}

pub fn button_system(
    mut state: ResMut<State<GameState>>,
    mut skill: ResMut<MainMenuSkillSelection>,
    mut interaction_query: Query<
        (&Interaction, &mut MainMenuSkillButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut skill_button) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                skill_button.is_clicked = true;
            }
            Interaction::Hovered => {
                if skill_button.is_clicked { // Finished a click while inside.
                    skill.0 = skill_button.skill;
                    let _ = state.set(GameState::LevelSelectionMenu);
                }
                skill_button.is_clicked = false;
            }
            Interaction::None => {
                skill_button.is_clicked = false; // They might have dragged outside while mousedown.
            }
        }
    }
}


// fn enter(
//     mut commands: Commands,
//     game_textures: Res<GameTextures>,
// ) {

// }

// fn update(
//     mut commands: Commands,
//     game_textures: Res<GameTextures>,
// ) {

// }

fn exit(
    mut commands: Commands,
    menu_components: Query<Entity, With<MainMenuComponent>>,
) {
    for e in menu_components.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn spawn_menu_logo(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
) {
    let scale = Vec3::new(TEXTURE_SCALE / 2., TEXTURE_SCALE / 2., 1.); // Logo is svga, so halve everything.
    const Y: f32 = 52.; // Make it overlap the background-tile-seam.
    commands
        .spawn_bundle(SpriteBundle {
            texture: game_textures.logo.clone(),
            transform: Transform{
                translation: Vec3::new(0., Y * POINT_SIZE, 1.), 
                scale,
                ..default()
            },        
            ..default()
        }).insert(MainMenuComponent);

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_textures.blink1.clone(),
            transform: Transform{
                translation: Vec3::new(-138. * POINT_SIZE, (Y + 1.) * POINT_SIZE, 2.),
                scale,
                ..default()
            },        
            ..default()
        }).insert(BlinkAnimationTimer{
            timer: Timer::from_seconds(FRAME_DURATION, true),
            index: -15,
            dwell: 30,
        }).insert(MainMenuComponent);

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_textures.blink2.clone(),
            transform: Transform{
                translation: Vec3::new(-26. * POINT_SIZE, (Y + 2.) * POINT_SIZE, 2.),
                scale,
                ..default()
            },        
            ..default()
        }).insert(BlinkAnimationTimer{
            timer: Timer::from_seconds(FRAME_DURATION, true),
            index: -22,
            dwell: 45,
        }).insert(MainMenuComponent);

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_textures.blink3.clone(),
            transform: Transform{
                translation: Vec3::new(94. * POINT_SIZE, (Y + 5.5) * POINT_SIZE, 2.),
                scale,
                ..default()
            },        
            ..default()
        }).insert(BlinkAnimationTimer{
            timer: Timer::from_seconds(FRAME_DURATION, true),
            index: -30,
            dwell: 37,
        }).insert(MainMenuComponent);

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_textures.blink3.clone(),
            transform: Transform{
                translation: Vec3::new(94. * POINT_SIZE, (Y + 5.5) * POINT_SIZE, 2.),
                scale,
                ..default()
            },        
            ..default()
        }).insert(BlinkAnimationTimer{
            timer: Timer::from_seconds(FRAME_DURATION, true),
            index: -30,
            dwell: 37,
        }).insert(MainMenuComponent);
}

fn spawn_menu_buttons(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
) {
    // Why do UI sizes need to be halved? Is this a retina thing that'll break on non-retina?
    commands.spawn_bundle(NodeBundle{
        style: Style{
            margin: UiRect::all(Val::Auto), // Center contents.
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        transform: Transform::from_xyz(0., 0., 2.),
        color: NORMAL_BUTTON.into(),
        ..default()
    }).insert(MainMenuComponent)
    .with_children(|outermost_parent| {
        outermost_parent.spawn_bundle(NodeBundle{
            style: Style{
                margin: UiRect::all(Val::Auto), // Center contents.
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        }).with_children(|parent| {
            parent.spawn_bundle(ButtonBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(10. * POINT_SIZE / 2.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: NORMAL_BUTTON.into(),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(120.0 * POINT_SIZE / 2.), Val::Px(61.0 * POINT_SIZE / 2.)),
                        ..default()
                    },
                    image: game_textures.exit_to_dos.clone().into(),
                    focus_policy: bevy::ui::FocusPolicy::Pass,
                    ..default()
                });
            });

            parent.spawn_bundle(ButtonBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(10. * POINT_SIZE / 2.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: NORMAL_BUTTON.into(),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(120.0 * POINT_SIZE / 2.), Val::Px(61.0 * POINT_SIZE / 2.)),
                        ..default()
                    },
                    image: game_textures.f4_settings.clone().into(),
                    focus_policy: bevy::ui::FocusPolicy::Pass,
                    ..default()
                });
            });
        });

        outermost_parent.spawn_bundle(NodeBundle{
            style: Style{
                margin: UiRect::all(Val::Auto), // Center contents.
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        }).with_children(|parent| {
            // Fun.
            parent.spawn_bundle(ButtonBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(10. * POINT_SIZE / 2.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: NORMAL_BUTTON.into(),
                ..default()
            })
            .insert(MainMenuSkillButton{
                is_clicked: false,
                skill: 0, // Fun.
            })
            .with_children(|parent| {
                parent.spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(120.0 * POINT_SIZE / 2.), Val::Px(61.0 * POINT_SIZE / 2.)),
                        margin: UiRect::all(Val::Auto), // Center contents.
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    image: game_textures.f1.clone().into(),
                    focus_policy: bevy::ui::FocusPolicy::Pass,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(ImageBundle {
                        style: Style {
                            size: Size::new(Val::Px(72.0 * POINT_SIZE / 2.), Val::Px(27.0 * POINT_SIZE / 2.)),
                            margin: UiRect::new(Val::Px(0.), Val::Px(0.), Val::Px(14. * POINT_SIZE / 2.), Val::Px(0.)),
                            ..default()
                        },
                        image: game_textures.fun.clone().into(),
                        focus_policy: bevy::ui::FocusPolicy::Pass,
                        ..default()
                    });            
                });
            });

            // Tricky.
            parent.spawn_bundle(ButtonBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(10. * POINT_SIZE / 2.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: NORMAL_BUTTON.into(),
                ..default()
            })
            .insert(MainMenuSkillButton{
                is_clicked: false,
                skill: 1,
            })
            .with_children(|parent| {
                parent.spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(120.0 * POINT_SIZE / 2.), Val::Px(61.0 * POINT_SIZE / 2.)),
                        margin: UiRect::all(Val::Auto), // Center contents.
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    image: game_textures.f2.clone().into(),
                    focus_policy: bevy::ui::FocusPolicy::Pass,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(ImageBundle {
                        style: Style {
                            size: Size::new(Val::Px(72.0 * POINT_SIZE / 2.), Val::Px(27.0 * POINT_SIZE / 2.)),
                            margin: UiRect::new(Val::Px(0.), Val::Px(0.), Val::Px(14. * POINT_SIZE / 2.), Val::Px(0.)),
                            ..default()
                        },
                        image: game_textures.tricky.clone().into(),
                        focus_policy: bevy::ui::FocusPolicy::Pass,
                        ..default()
                    });            
                });
            });

            // Taxing.
            parent.spawn_bundle(ButtonBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(10. * POINT_SIZE / 2.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: NORMAL_BUTTON.into(),
                ..default()
            })
            .insert(MainMenuSkillButton{
                is_clicked: false,
                skill: 2,
            })
            .with_children(|parent| {
                parent.spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(120.0 * POINT_SIZE / 2.), Val::Px(61.0 * POINT_SIZE / 2.)),
                        margin: UiRect::all(Val::Auto), // Center contents.
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    image: game_textures.f3.clone().into(),
                    focus_policy: bevy::ui::FocusPolicy::Pass,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(ImageBundle {
                        style: Style {
                            size: Size::new(Val::Px(72.0 * POINT_SIZE / 2.), Val::Px(27.0 * POINT_SIZE / 2.)),
                            margin: UiRect::new(Val::Px(0.), Val::Px(0.), Val::Px(14. * POINT_SIZE / 2.), Val::Px(0.)),
                            ..default()
                        },
                        image: game_textures.taxing.clone().into(),
                        focus_policy: bevy::ui::FocusPolicy::Pass,
                        ..default()
                    });            
                });
            });

            // Mayhem.
            parent.spawn_bundle(ButtonBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(10. * POINT_SIZE / 2.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: NORMAL_BUTTON.into(),
                ..default()
            })
            .insert(MainMenuSkillButton{
                is_clicked: false,
                skill: 3,
            })
            .with_children(|parent| {
                parent.spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(120.0 * POINT_SIZE / 2.), Val::Px(61.0 * POINT_SIZE / 2.)),
                        margin: UiRect::all(Val::Auto), // Center contents.
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    image: game_textures.level_rating.clone().into(),
                    focus_policy: bevy::ui::FocusPolicy::Pass,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(ImageBundle {
                        style: Style {
                            size: Size::new(Val::Px(72.0 * POINT_SIZE / 2.), Val::Px(27.0 * POINT_SIZE / 2.)),
                            margin: UiRect::new(Val::Px(0.), Val::Px(0.), Val::Px(14. * POINT_SIZE / 2.), Val::Px(0.)),
                            ..default()
                        },
                        image: game_textures.mayhem.clone().into(),
                        focus_policy: bevy::ui::FocusPolicy::Pass,
                        ..default()
                    });            
                });
            });
        });

        outermost_parent.spawn_bundle(NodeBundle{ // This pushes things down under the logo.
            style: Style {
                size: Size::new(Val::Px(0.), Val::Px(100.0 * POINT_SIZE / 2.)),
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        });
    });
}
