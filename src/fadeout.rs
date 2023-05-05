// Based on: github.com/mwbryant/rpg-bevy-tutorial
use bevy::prelude::*;
use crate::{GameState, GameTextures};

pub struct FadeoutPlugin;

// This is a component not a resource because it doesn't exist when a fade isn't running.
#[derive(Component)]
struct ScreenFade {
    alpha: f32,
    sent: bool, // Has it switched over to next_state yet?
    next_state: GameState,
    timer: Timer,
}

// This is a resource for efficient lookup, as it is used as a run condition.
#[derive(Resource)]
pub struct ScreenFadeIsTransitioning(bool);

impl Plugin for FadeoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(fadeout);
        app.insert_resource(ScreenFadeIsTransitioning(false));
    }
}

// To be used as a run condition using 'run_if' for any interaction system you want disabled
// while transitioning.
pub fn screen_fade_is_not_transitioning(is_transitioning: Res<ScreenFadeIsTransitioning>) -> bool {
    !is_transitioning.0
}

fn fadeout(
    mut commands: Commands,
    mut fade_query: Query<(Entity, &mut ScreenFade, &mut Sprite)>,
    mut state: ResMut<NextState<GameState>>,
    mut is_transitioning: ResMut<ScreenFadeIsTransitioning>,
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
            is_transitioning.0 = false;
        }
    }
}

// TODO consider the pros and cons of making fadeouts happen using Events rather than calling this directly.
pub fn create_fadeout(
    commands: &mut Commands,
    next_state: GameState,
    game_textures: &Res<GameTextures>,
    mut is_transitioning: ResMut<ScreenFadeIsTransitioning>,
) {
    // Stop eg mouse clicks while transitioning.
    is_transitioning.0 = true;
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
