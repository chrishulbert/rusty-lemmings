// Based on: github.com/mwbryant/rpg-bevy-tutorial
use bevy::prelude::*;
use crate::{GameState, GameTextures};

pub struct FadeoutPlugin;

#[derive(Component)]
struct ScreenFade {
    alpha: f32,
    sent: bool, // Has it switched over to next_state yet?
    next_state: GameState,
    timer: Timer,
}

impl Plugin for FadeoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(fadeout);
    }
}

fn fadeout(
    mut commands: Commands,
    mut fade_query: Query<(Entity, &mut ScreenFade, &mut Sprite)>,
    mut state: ResMut<NextState<GameState>>,
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

        if fade.timer.percent() > 0.5 && !fade.sent {
            state.set(fade.next_state);
            fade.sent = true;
        }

        if fade.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn create_fadeout(
    commands: &mut Commands,
    next_state: GameState,
    game_textures: &Res<GameTextures>,
    //mut state: ResMut<State<GameState>>,
) {
    // TODO now that bevy 0.10 does not have the state stack, I have to rethink
    // some way to replicate how it used to 'push' a fading state to disable the old screen
    // yet still have it displayed.
    //_ = state.push(GameState::Fading); // So you can't tap anything on the old screen.
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                 color: Color::NONE,
                 custom_size: Some(Vec2::splat(100000.0)),
                ..Default::default()
            },
            texture: game_textures.white.clone(),
            transform: Transform {
                translation: Vec3::new(0., 0., 998.0), // 999 is mouse, we want to be under that.
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ScreenFade {
            alpha: 0.,
            sent: false,
            next_state: next_state,
            timer: Timer::from_seconds(1., TimerMode::Once),
        });
}
