// Based on: github.com/mwbryant/rpg-bevy-tutorial
use bevy::prelude::*;
use crate::{GameState, GameTextures};

pub struct FadePlugin;

#[derive(Component)]
struct ScreenFade {
    alpha: f32,
    has_changed_state: bool,
    next_state: GameState,
    timer: Timer,
}

impl Plugin for FadePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(fadeout);//.add_system(ui_fadeout);
    }
}

// fn ui_fadeout(
//     fade_query: Query<&ScreenFade>,
//     mut ui_query: Query<&mut UiColor>,
//     mut text_query: Query<&mut Text>,
// ) {
//     if let Some(fade) = fade_query.iter().next() {
//         for mut ui_color in ui_query.iter_mut() {
//             ui_color.0.set_a(1.0 - fade.alpha);
//         }
//         for mut text in text_query.iter_mut() {
//             for section in text.sections.iter_mut() {
//                 section.style.color.set_a(1.0 - fade.alpha);
//             }
//         }
//     }
// }

fn fadeout(
    mut commands: Commands,
    mut fade_query: Query<(Entity, &mut ScreenFade, &mut Sprite)>,
    mut state: ResMut<State<GameState>>,
    time: Res<Time>,
) {
    for (entity, mut fade, mut sprite) in fade_query.iter_mut() {
        fade.timer.tick(time.delta());
        if fade.timer.percent() < 0.5 {
            fade.alpha = fade.timer.percent() * 2.0;
        } else {
            fade.alpha = fade.timer.percent_left() * 2.0;
        }
        sprite.color.set_a(fade.alpha);

        if fade.timer.percent() > 0.5 && !fade.has_changed_state {
            _ = state.set(fade.next_state); // Replace could be used instead of set if we pushed a 'fade' state at start.
            fade.has_changed_state = true;
        }

        if fade.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn create_fade(
    commands: &mut Commands,
    next_state: GameState,
    game_textures: &Res<GameTextures>,
) {
    let x = commands.spawn_bundle(NodeBundle{
        style: Style{
            margin: UiRect::all(Val::Auto), // Center contents.
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            size: Size::new(Val::Px(9999.0), Val::Px(9999.0)),
            ..default()
        },
        transform: Transform::from_xyz(0., 0., 999.),
        global_transform: GlobalTransform::from_xyz(0., 0., 999.),
        color: Color::PURPLE.into(),
        ..default()
    });
    // .insert(ScreenFade {
    //     alpha: 0.0,
    //     has_changed_state: false,
    //     next_state: next_state,
    //     timer: Timer::from_seconds(1.0, false),
    // })
    // .with_children(|parent| {
    //     parent.spawn_bundle(ImageBundle {
    //         style: Style {
    //             size: Size::new(Val::Px(9999.0), Val::Px(9999.0)),
    //             ..default()
    //         },
    //         transform: Transform::from_xyz(0., 0., 999.),
    //         global_transform: GlobalTransform::from_xyz(0., 0., 999.),
    //         image: game_textures.black.clone().into(),
    //         color: Color::RED.into(),
    //         ..default()
    //     });
    // });

    // commands
    //     .spawn_bundle(SpriteBundle {
    //         sprite: Sprite{
    //             color: Color::rgba(0., 0., 0., 0.),
    //             custom_size: Some(Vec2::splat(100000.0)),
    //             ..Default::default()
    //         },
    //         texture: game_textures.black.clone(),
    //         transform: Transform {
    //             translation: Vec3::new(0.0, 0.0, 999.0),
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     })
    //     .insert(ScreenFade {
    //         alpha: 0.0,
    //         has_changed_state: false,
    //         next_state: next_state,
    //         timer: Timer::from_seconds(1.0, false),
    //     });
}
