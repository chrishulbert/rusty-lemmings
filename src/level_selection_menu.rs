use bevy::prelude::*;
use crate::{GameTextures, GameState, POINT_SIZE};
use crate::menu_common::{NORMAL_BUTTON, spawn_menu_background, button_highlight_system};

pub struct MainMenuSkillSelection(isize);

pub struct LevelSelectionMenuPlugin;

impl Plugin for LevelSelectionMenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MainMenuSkillSelection(0));
        app.add_system_set(
            SystemSet::on_enter(GameState::LevelSelectionMenu)
                // .with_system(enter)
                .with_system(spawn_levels)
                .with_system(spawn_menu_background)
        );
        app.add_system_set(
            SystemSet::on_update(GameState::LevelSelectionMenu)
                // .with_system(update)
                .with_system(button_highlight_system)
        );
        // .add_system_set(
        //     SystemSet::on_exit(GameState::LevelSelectionMenu)
        //         .with_system(exit),
        // )
        // .add_system(button_system);
	}
}

fn spawn_text(text: &str, parent: &mut ChildBuilder, game_textures: Res<GameTextures>) {
    let style = Style {
        size: Size::new(Val::Px(16.0 * POINT_SIZE / 2.), Val::Px(16.0 * POINT_SIZE / 2.)),
        ..default()
    };
    for c in text.chars() {
        let a = c as u32;
        if 33 <= a && a <= 126 { // Menu font is '!'(33) - '~'(126)
            let index = (a - 33) as usize;
            parent.spawn_bundle(ImageBundle {
                style: style.clone(),
                image: game_textures.menu_font[index].clone().into(),
                focus_policy: bevy::ui::FocusPolicy::Pass,
                ..default()
            });
        } else if a == 32 { // Space.
            parent.spawn_bundle(NodeBundle {
                style: style.clone(),
                color: Color::NONE.into(),
                focus_policy: bevy::ui::FocusPolicy::Pass,
                ..default()
            });
        }
    }
}

fn spawn_level_button(parent: &mut ChildBuilder, game_textures: Res<GameTextures>) {
    parent.spawn_bundle(ButtonBundle {
        style: Style {
            padding: UiRect::all(Val::Px(10. * POINT_SIZE / 2.)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        color: NORMAL_BUTTON.into(),
        ..default()
    }).with_children(|parent| {
        parent.spawn_bundle(NodeBundle{ // Do we need this? Or could letters be direct children of the button? Is this like div-itis?
            style: Style{
                margin: UiRect::all(Val::Auto), // Center contents.
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            focus_policy: bevy::ui::FocusPolicy::Pass,
            ..default()
        }).with_children(|parent| {
            spawn_text("The quick brown fox!", parent, game_textures);
        }); // End of spawning the letters inside a container.
    }); // End of spawning the button.
}

fn spawn_levels(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    // skill_selection: Res<MainMenuSkillSelection>,
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
    }).with_children(|outermost_parent| {
        outermost_parent.spawn_bundle(NodeBundle{
            style: Style{
                margin: UiRect::all(Val::Auto), // Center contents.
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        }).with_children(|parent| {
            spawn_level_button(parent, game_textures);
        });
    });
}

// fn enter(
//     mut commands: Commands,
//     game_textures: Res<GameTextures>,
// ) {
// }
