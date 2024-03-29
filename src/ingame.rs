use std::time::Duration;
use std::collections::HashMap;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::render::render_resource::Extent3d;
use crate::{GameTextures, GameState, TEXTURE_SCALE, SCALE, POINT_SIZE, FPS};
use crate::lemmings::models::{Game, ObjectInfo};
use crate::level_preview::LevelSelectionResource;
use crate::lemmings::level_renderer;
use crate::helpers::{multi_scale, u32_to_rgba_u8};
use crate::helpers::{make_image_from_bitmap, make_atlas_from_animation};
use crate::{ORIGINAL_GAME_W, FRAME_DURATION};
use crate::lemmings::sizes;
use crate::mouse_cursor::{MouseCursorShouldBecomeSelectorEvent, update_mouse_cursor_style_system, reset_mouse_cursor_system};

const DROP_POINTS_PER_FRAME: f32 = 2.;
const LEMMING_NOMINAL_HEIGHT_HALF: i32 = 5; // Usual height for a lemming sprite in game points. Halved for use later.
const LEMMING_WIDTH_FOR_BASE: f32 = 3.; // How many points under it to check to see if any land exists.

pub struct InGamePlugin;

/// This does a sort of double-duty as both the skill panel buttons, and as 'skill'.
/// I did debate the correctness of using this for skills when it contains the +-/pause/nuke buttons.
/// But if i made a separate enum for skills, that'd be more correct, but would involve doubling up and mapping.
#[derive(Eq, Hash, Debug, PartialEq, Clone, Copy)]
enum SkillPanelSelection {
    SpeedMinus = 0, // Momentary.
    SpeedPlus = 1, // Momentary.
    Climb = 2,
    Umbrella = 3,
    Explode = 4,
    Block = 5,
    Build = 6,
    Bash = 7,
    MineDiagonal = 8,
    DigVertical = 9,
    Pause = 10,
    Nuke = 11, // Momentary.
}

impl SkillPanelSelection {
    fn from_index(i: isize) -> Option<SkillPanelSelection> {
        match i {
            0 => Some(SkillPanelSelection::SpeedMinus),
            1 => Some(SkillPanelSelection::SpeedPlus),
            2 => Some(SkillPanelSelection::Climb),
            3 => Some(SkillPanelSelection::Umbrella),
            4 => Some(SkillPanelSelection::Explode),
            5 => Some(SkillPanelSelection::Block),
            6 => Some(SkillPanelSelection::Build),
            7 => Some(SkillPanelSelection::Bash),
            8 => Some(SkillPanelSelection::MineDiagonal),
            9 => Some(SkillPanelSelection::DigVertical),
            10 => Some(SkillPanelSelection::Pause),
            11 => Some(SkillPanelSelection::Nuke),
            _ => None,
        }
    }
}

/// Resource.
#[derive(Resource)]
struct GameTimer(Timer);

/// Resource.
#[derive(Resource)]
struct InGameStartCountdown(i32); // Countdown to the entrance opening.
#[derive(Resource)]
struct InGameDropCountdown(i32); // Countdown between dropping lemmings. -1 if hasn't started yet, or has dropped all lemmings.
#[derive(Resource)]
struct InGameLemmingsContainerId(Entity); // The entity id of the lemmings container.
#[derive(Resource)]
struct InGameSlices(Option<Slices>);
#[derive(Resource)]
struct InGameBottomPanelId(Entity); // The id of the skill selection / map panel.
#[derive(Resource)]
struct InGamePanelDigits(Vec<LeftRightEntityPair>);
#[derive(Resource)]
struct InGameSkillSelectionIndicatorId(Entity); // The id of the skill selection indicator.
#[derive(Resource)]
struct InGameSpeedSelectionIndicatorId(Entity); // The id of the speed selection indicator.
#[derive(Resource)]
struct InGamePauseSelectionIndicatorId(Entity); // The id of the pause selection indicator.
#[derive(Resource)]
struct InGameNukeSelectionIndicatorId(Entity); // The id of the nuke selection indicator.
#[derive(Resource)]
struct InGameSkillSelection(Option<SkillPanelSelection>);
#[derive(Resource)]
struct InGameSkillCounts(HashMap<SkillPanelSelection, isize>);
#[derive(Resource)]
struct InGameReleaseRate(isize); // Current release rate 0-99.
#[derive(Resource)]
struct InGameIsPaused(bool);

// Even though we refer to some entities by Id, we have to give them components so bevy doesn't panic when
// querying 2+ of them in the one func.
#[derive(Component)]
struct InGameBottomPanelComponent;
#[derive(Component)]
struct InGamePanelDigitComponent;
#[derive(Component)]
struct InGameSkillSelectionIndicatorComponent;
#[derive(Component)]
struct InGameSpeedSelectionIndicatorComponent;
#[derive(Component)]
struct InGamePauseSelectionIndicatorComponent;
#[derive(Component)]
struct InGameNukeSelectionIndicatorComponent;

// Events.
struct LemmingUnderPointerEvent(Option<Entity>);

impl Plugin for InGamePlugin {
	fn build(&self, app: &mut App) {
        // Instead of timers per entity, we use a global timer so that everyone moves in unison.
        app.insert_resource(GameTimer(Timer::from_seconds(FRAME_DURATION, TimerMode::Repeating)));
        app.insert_resource(InGameStartCountdown(FPS as i32));
        app.insert_resource(InGameDropCountdown(-1));
        app.insert_resource(InGameLemmingsContainerId(Entity::from_raw(0)));
        app.insert_resource(InGameSlices(None));
        app.insert_resource(InGameBottomPanelId(Entity::from_raw(0)));
        app.insert_resource(InGamePanelDigits(vec![]));
        app.insert_resource(InGameSkillSelectionIndicatorId(Entity::from_raw(0)));
        app.insert_resource(InGameSpeedSelectionIndicatorId(Entity::from_raw(0)));
        app.insert_resource(InGamePauseSelectionIndicatorId(Entity::from_raw(0)));
        app.insert_resource(InGameNukeSelectionIndicatorId(Entity::from_raw(0)));
        app.insert_resource(InGameSkillSelection(None));
        app.insert_resource(InGameSkillCounts(HashMap::new()));
        app.insert_resource(InGameReleaseRate(50));
        app.insert_resource(InGameIsPaused(false));
        app.add_event::<UpdatePanelDigitsEvent>();
        app.add_event::<LemmingUnderPointerEvent>();

		app.add_systems((
            enter,
            enter_and_spawn_bottom_skill_panel,
        ).in_schedule(OnEnter(GameState::InGame)));

        // Systems on the same line were in parallel before bevy 0.10 changed the syntax, it's poorly documented,
        // so i'm not sure how to reproduce ordered groups of parallel systems. Perhaps, efficiency-wise, it
        // doesn't matter.
        app.add_systems((
            tick,            
            scroll, determine_lemming_under_mouse_system,
            mouse_click_system, do_countdown, drop_lemmings,
            update_objects, update_lemmings,
            update_panel_digits_system, update_mouse_cursor_style_system,
        ).chain().in_set(OnUpdate(GameState::InGame)));

        app.add_systems((
            exit,
            reset_mouse_cursor_system,
        ).in_schedule(OnExit(GameState::InGame)));
	}
}

#[derive(Component)]
struct InGameComponent; // Marker component so it can be despawned.

#[derive(Component)]
struct ObjectComponent {
    pub info: ObjectInfo,
}

#[derive(Component)]
struct LemmingComponent {
    is_facing_right: bool,
    skill_in_use: Option<SkillPanelSelection>,
    has_umbrella: bool,
    can_climb: bool,
    builder_bricks_remaining: i8,
}

impl Default for LemmingComponent {
    fn default() -> LemmingComponent {
        LemmingComponent {
            is_facing_right: true,
            skill_in_use: None,
            has_umbrella: false,
            can_climb: false,
            builder_bricks_remaining: 0,
        }
    }
}

#[derive(Component)]
struct MapContainerComponent { // Controls the x/y scroll of the map.
    pub min_x: f32, // In bevy transform coords.
    pub max_x: f32,
}

fn exit(
    mut commands: Commands,
    menu_components: Query<Entity, With<InGameComponent>>,
) {
    for e in menu_components.iter() {
        commands.entity(e).despawn_recursive();
    }
}

struct SliceWithoutHandle {
    pub bitmap: Vec<u32>,
    pub x: isize, // Scaled-up pixels. This includes the level's min_x.
    pub width: usize,
    pub height: usize,
}

// N pixels in the bitmap, not display points or original lemmings pixels.
// Probably a good number would be the size of a lemming (16px) x scale (12), so each slice is as big as a lemming.
const SLICE_WIDTH: usize = 192;

// Slice a map into pieces of N width.
// The image should be scaled.
fn slice(image: &[u32], width: usize, height: usize, min_x: isize) -> Vec<SliceWithoutHandle> {
    let mut slices = Vec::<SliceWithoutHandle>::with_capacity(width / SLICE_WIDTH + 1);
    let mut offset_x: usize = 0;
    while offset_x < width {
        let remaining_cols = width - offset_x;
        let this_width = std::cmp::min(SLICE_WIDTH, remaining_cols);
        let mut slice_bitmap = Vec::<u32>::with_capacity(this_width * height);
        for y in 0..height {
            let input_offset = y * width + offset_x;
            for x in 0..this_width {
                slice_bitmap.push(image[input_offset + x]);
            }
        }
        slices.push(SliceWithoutHandle{bitmap: slice_bitmap, x: (offset_x as isize) + min_x, width: this_width, height});
        offset_x += SLICE_WIDTH;
    }
    slices
}

struct Slices {
    slices: Vec<Slice>,
    x_to_slice_index_lookup: HashMap<i32, u32>, // eg x_to_slice_index[x] = index in slices.
}

struct Slice {
    pub bitmap: Vec<u32>,
    pub texture: Handle<Image>,

    // The following are in scaled-up pixels:
    pub x: isize, 
    pub width: usize,
    pub height: usize,

    // In game points:
    game_points_x: isize,
    game_points_width: usize,
}

fn convert_slices_to_bevy(in_slices: Vec<SliceWithoutHandle>, images: &mut ResMut<Assets<Image>>) -> Slices {
    let slices: Vec<Slice> = in_slices.into_iter().map(|s| {
        let u8_data = u32_to_rgba_u8(&s.bitmap);
        let image = Image::new(Extent3d{width: s.width as u32, height: s.height as u32, depth_or_array_layers: 1},
            bevy::render::render_resource::TextureDimension::D2,
            u8_data,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb);    
        let texture = images.add(image);
        Slice{
            bitmap: s.bitmap,
            texture,
            x: s.x,
            width: s.width,
            height: s.height,
            game_points_x: s.x / SCALE as isize,
            game_points_width: s.width / SCALE,
        }
    }).collect();

    // Make a lookup so its easy to find the slices.
    // Not using a simple array because the x positions might be negative.
    let mut x_to_slice_index_lookup = HashMap::<i32, u32>::new();
    for (slice_index, slice) in slices.iter().enumerate() {
        for delta in 0..slice.game_points_width {
            x_to_slice_index_lookup.insert(slice.game_points_x as i32 + delta as i32, slice_index as u32);
        }
    }

    Slices {
        slices,
        x_to_slice_index_lookup,
    }
}

// Drop lemmings every now and again.
fn drop_lemmings(
	mut commands: Commands,
	game_textures: Res<GameTextures>,
    timer: Res<GameTimer>,
    mut drop_countdown: ResMut<InGameDropCountdown>,
    query: Query<(&Transform, &ObjectComponent)>,
    lemmings_container_id: Res<InGameLemmingsContainerId>,
    is_paused: Res<InGameIsPaused>,
    release_rate: Res<InGameReleaseRate>,
) {
    if is_paused.0 { return }
    if drop_countdown.0 < 0 { return } // hasn't started yet or is complete.
    if timer.0.just_finished() {
        let new_countdown = drop_countdown.0 - 1;
        if new_countdown <= 0 {
            // For each entrance, drop 1 lemming, until we've dropped enough lemmings.
            for (t, o) in query.iter() {
                let t: &Transform = t;
                let o: &ObjectComponent = o;
                if o.info.is_entrance {
                    // TODO only spawn if we haven't run out of lemmings to spawn, otherwise break the for.
                    spawn_a_lemming(&mut commands, &t.translation, &game_textures, &lemmings_container_id.0);
                }
            }
            // 99 = 4 frames, 50 = about 2s.
            drop_countdown.0 = ((100 - release_rate.0 as i32) * FPS as i32 / 25).max(4);
        } else {
            drop_countdown.0 = new_countdown;
        }
    }
}

// Spawn a lemming right now.
fn spawn_a_lemming(
    commands: &mut Commands,
    entrance: &Vec3,
    game_textures: &Res<GameTextures>,
    lemmings_container_id: &Entity,
) {
    commands.entity(*lemmings_container_id).with_children(|parent| {
        parent.spawn(SpriteSheetBundle{
            texture_atlas: game_textures.falling_right.clone(),
            transform: Transform{
                scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
                translation: Vec3::new(round_to_nearest_point(entrance.x), round_to_nearest_point(entrance.y), 0.),
                ..default()
            },
            ..default()
        }).insert(LemmingComponent::default());
    });
}

fn update_objects(
    timer: Res<GameTimer>,
    start_countdown: Res<InGameStartCountdown>,
    mut drop_countdown: ResMut<InGameDropCountdown>,
    mut query: Query<(
        &mut TextureAtlasSprite,
        &ObjectComponent,
    )>,
    is_paused: Res<InGameIsPaused>,
) {
    if is_paused.0 { return }
    if timer.0.just_finished() {
        for (mut tas, object_unknown) in &mut query {
            let object: &ObjectComponent = object_unknown; // Otherwise RLS can't suggest the type.
            if object.info.is_entrance {
                // Entrance is a special case: has to wait for the start countdown.
                if start_countdown.0 <= 0 {
                    if tas.index > 0 { // Not fully open yet.
                        let new_index = tas.index + 1;
                        if new_index >= object.info.frame_count as usize {
                            tas.index = 0; // Full open now.
                            drop_countdown.0 = (FPS / 2.) as i32; // Wait half a sec to drop.
                        } else {
                            tas.index = new_index;
                        }    
                    }
                }
            } else {
                tas.index = (tas.index + 1) % (object.info.frame_count as usize);
            }
        }
    }
}

fn do_countdown(
    timer: Res<GameTimer>,
    mut start_countdown: ResMut<InGameStartCountdown>,
) {
    if start_countdown.0 > 0 {
        if timer.0.just_finished() {
            start_countdown.0 -= 1;
        }    
    }
}

fn tick(
    time: Res<Time>,
    mut timer: ResMut<GameTimer>,
) {
    timer.0.tick(time.delta());
}

fn determine_lemming_under_mouse_system(
    windows: Query<&Window>,
    map_query: Query<&Transform, &MapContainerComponent>,
    lemmings_query: Query<(Entity, &Transform), &LemmingComponent>,
    mut event: EventWriter<LemmingUnderPointerEvent>,
    mut mouse_should_become_selector: EventWriter<MouseCursorShouldBecomeSelectorEvent>,
) {
    let closest = determine_lemming_under_mouse(windows, map_query, lemmings_query);
    event.send(LemmingUnderPointerEvent(closest));    
    mouse_should_become_selector.send(MouseCursorShouldBecomeSelectorEvent(closest.is_some()));
}

fn determine_lemming_under_mouse(
    windows: Query<&Window>,
    map_query: Query<&Transform, &MapContainerComponent>,
    lemmings_query: Query<(Entity, &Transform), &LemmingComponent>,
) -> Option<Entity> {
    let Some(window) = windows.iter().next() else { return None };
    let Some(position) = window.cursor_position() else { return None };
    let Ok(map) = map_query.get_single() else { return None };

    // Locate the mouse.
    let mouse_x = position.x - window.width() / 2.;
    let mouse_y = position.y - window.height() / 2.; // +=top, -=bottom.
    let mouse_x_in_game = mouse_x - map.translation.x;
    let mouse_y_in_game = mouse_y - map.translation.y; // +=top, -=bot, 0=mid of game area.

    // Find closest lemming.
    let min_delta = 10. * POINT_SIZE;
    let mut closest: Option<Entity> = None;
    let mut closest_delta: f32 = 99999.;
    for (e, &t) in lemmings_query.iter() {
        let dx = (t.translation.x - mouse_x_in_game).abs();
        let dy = (t.translation.y - mouse_y_in_game).abs();
        if dx <= min_delta && dy <= min_delta {
            let this_delta = dx + dy;
            if this_delta < closest_delta {
                closest = Some(e);
                closest_delta = this_delta;
            }
        }
    }
    closest
}

/// Scroll left and right if your mouse is at the edge.
fn scroll(
    time: Res<Time>,
    windows: Query<&Window>,
    mut query: Query<(&mut Transform, &MapContainerComponent)>,
) {
    let Some(window) = windows.iter().next() else { return };
    let Some(position) = window.cursor_position() else { return };
    
    let delta: isize;
    if position.x < window.width() * 0.05 {
        delta = 2;
    } else if position.x < window.width() * 0.1 {
        delta = 1;
    } else if position.x > window.width() * 0.95 {
        delta = -2;
    } else if position.x > window.width() * 0.9 {
        delta = -1;
    } else {
        delta = 0;
    }
    if delta != 0 {
        for (mut transform, container) in query.iter_mut() {
            let new_x = transform.translation.x + (delta as f32 * time.delta().as_secs_f32() * window.width() * 0.3).round();
            let clamped_x = new_x.min(container.max_x).max(container.min_x);
            transform.translation.x = clamped_x;
        }
    }
}

fn mouse_click_system(
    windows: Query<&Window>,
    mouse_button_input: Res<Input<MouseButton>>,
    bottom_panel_id: Res<InGameBottomPanelId>,
    bottom_panel_query: Query<&Transform, With<InGameBottomPanelComponent>>,
    (
        skill_selection_indicator_id,
        speed_selection_indicator_id,
        pause_selection_indicator_id,
        nuke_selection_indicator_id): (
            Res<InGameSkillSelectionIndicatorId>,
            Res<InGameSpeedSelectionIndicatorId>,
            Res<InGamePauseSelectionIndicatorId>,
            Res<InGameNukeSelectionIndicatorId>
        ),
    (
        mut skill_selection_indicator_query,
        mut pause_selection_indicator_query,
        mut speed_selection_indicator_query,
        mut nuke_selection_indicator_query): (
            Query<&mut Transform, (With<InGameSkillSelectionIndicatorComponent>, Without<InGameBottomPanelComponent>)>,
            Query<&mut Transform, (With<InGamePauseSelectionIndicatorComponent>, Without<InGameBottomPanelComponent>, Without<InGameSkillSelectionIndicatorComponent>)>,
            Query<&mut Transform, (With<InGameSpeedSelectionIndicatorComponent>, Without<InGameBottomPanelComponent>, Without<InGameSkillSelectionIndicatorComponent>, Without<InGamePauseSelectionIndicatorComponent>)>,
            Query<&mut Transform, (With<InGameNukeSelectionIndicatorComponent>, Without<InGameBottomPanelComponent>, Without<InGameSkillSelectionIndicatorComponent>, Without<InGamePauseSelectionIndicatorComponent>, Without<InGameSpeedSelectionIndicatorComponent>)>
        ),
    (mut in_game_skill_selection, mut in_game_skill_counts): (ResMut<InGameSkillSelection>, ResMut<InGameSkillCounts>),
    mut is_paused: ResMut<InGameIsPaused>,
    mut release_rate: ResMut<InGameReleaseRate>,
    mut update_panel_digits_events: EventWriter<UpdatePanelDigitsEvent>,
    mut lemming_under_pointer: EventReader<LemmingUnderPointerEvent>,
    mut lemmings_query: Query<&mut LemmingComponent>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let Some(window) = windows.iter().next() else { return };
        let Some(position) = window.cursor_position() else { return };
        let mouse_x = position.x - window.width() / 2.;
        let mouse_y = position.y - window.height() / 2.;

        let Ok(bottom_panel) = bottom_panel_query.get(bottom_panel_id.0) else { return };
        let bottom_panel_top = bottom_panel.translation.y + sizes::SKILL_PANEL_CLICKABLE_HEIGHT as f32 * POINT_SIZE;
        let did_click_bottom_panel = mouse_y <= bottom_panel_top;
        if did_click_bottom_panel {
            let bottom_panel_click_x_position_pt = (mouse_x - bottom_panel.translation.x) / POINT_SIZE + sizes::SKILL_PANEL_WIDTH as f32 / 2.;
            if bottom_panel_click_x_position_pt >= 0. {
                let button_index = bottom_panel_click_x_position_pt as isize / sizes::SKILL_PANEL_BUTTON_WIDTH as isize;
                if let Some(selection) = SkillPanelSelection::from_index(button_index) {
                    let leftmost_skill: f32 = -9. * sizes::SKILL_PANEL_BUTTON_WIDTH as f32 - 7.5;
                    match selection {
                        SkillPanelSelection::SpeedMinus | SkillPanelSelection::SpeedPlus => {
                            let delta: isize = if selection == SkillPanelSelection::SpeedMinus { -10 } else { 10 };
                            release_rate.0 += delta;
                            if release_rate.0 % 10 != 0 { release_rate.0 = ((release_rate.0 as f32 / 10.).round() as isize) * 10 } 
                            if release_rate.0 < 1 { release_rate.0 = 1 }
                            if release_rate.0 > 99 { release_rate.0 = 99 }
                            update_panel_digits_events.send(UpdatePanelDigitsEvent);
                            let index: f32 = if selection == SkillPanelSelection::SpeedMinus { 0. } else { 1. };
                            if let Ok(mut speed_indicator) = speed_selection_indicator_query.get_mut(speed_selection_indicator_id.0) {
                                speed_indicator.translation = Vec3::new(
                                    (leftmost_skill + index * sizes::SKILL_PANEL_BUTTON_WIDTH as f32) * POINT_SIZE / TEXTURE_SCALE,
                                    0.,
                                    11.);
                            }
                        },
                        SkillPanelSelection::Pause => {
                            is_paused.0 ^= true; // No selection indicator because you can be both paused and have a skill chosen.
                            if let Ok(mut pause_indicator) = pause_selection_indicator_query.get_mut(pause_selection_indicator_id.0) {
                                if is_paused.0 {
                                    pause_indicator.translation = Vec3::new(
                                        (leftmost_skill + 10. * sizes::SKILL_PANEL_BUTTON_WIDTH as f32) * POINT_SIZE / TEXTURE_SCALE,
                                        0.,
                                        11.);
                                } else {
                                    pause_indicator.translation = Vec3::new(-99999., 0., 11.);
                                }    
                            }
                        },
                        SkillPanelSelection::Nuke => {
                            if let Ok(mut nuke_indicator) = nuke_selection_indicator_query.get_mut(nuke_selection_indicator_id.0) {
                                nuke_indicator.translation = Vec3::new(
                                    (leftmost_skill + 11. * sizes::SKILL_PANEL_BUTTON_WIDTH as f32) * POINT_SIZE / TEXTURE_SCALE,
                                    0.,
                                    11.);
                            }
                        },
                        _ => { // Normal skill.
                            in_game_skill_selection.0 = Some(selection);
                            if let Ok(mut skill_selection_indicator) = skill_selection_indicator_query.get_mut(skill_selection_indicator_id.0) {
                                skill_selection_indicator.translation = Vec3::new(
                                    (leftmost_skill + button_index as f32 * sizes::SKILL_PANEL_BUTTON_WIDTH as f32) * POINT_SIZE / TEXTURE_SCALE, 
                                    0.,
                                    11.);
                            }                    
                        },
                    }
                }
            }
        } else { // Didn't click the bottom panel, clicked on the game.
            let Some(event) = lemming_under_pointer.iter().next() else { return };
            let Some(lemming_id) = event.0 else { return };
            let Ok(mut lemming) = lemmings_query.get_mut(lemming_id) else { return };
            let Some(selected_skill) = in_game_skill_selection.0 else { return };
            if lemming.skill_in_use == Some(selected_skill) { return };
            let Some(skill_count) = in_game_skill_counts.0.get_mut(&selected_skill) else { return };
            if *skill_count <= 0 { return };
            *skill_count -= 1;
            update_panel_digits_events.send(UpdatePanelDigitsEvent);
            lemming.skill_in_use = Some(selected_skill);
        }
    } else if mouse_button_input.just_released(MouseButton::Left) { // Release the momentaries if any.
        if let Ok(mut speed_indicator) = speed_selection_indicator_query.get_mut(speed_selection_indicator_id.0) {
            speed_indicator.translation = Vec3::new(99999., 0., 0.);
        }
        if let Ok(mut nuke_indicator) = nuke_selection_indicator_query.get_mut(nuke_selection_indicator_id.0) {
            nuke_indicator.translation = Vec3::new(99999., 0., 0.);
        }
        // TODO if they clicked a skill that has 0 available, deselect it on mouseup.
    }
}

fn enter(
    level_selection: Res<LevelSelectionResource>,
	game: Res<Game>,
	windows: Query<&Window>,
	mut commands: Commands,
    mut timer: ResMut<GameTimer>,
    mut start_countdown: ResMut<InGameStartCountdown>,
    mut drop_countdown: ResMut<InGameDropCountdown>,
	mut images: ResMut<Assets<Image>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut lemmings_container_id: ResMut<InGameLemmingsContainerId>,
    mut slices_resource: ResMut<InGameSlices>,
    mut release_rate: ResMut<InGameReleaseRate>,
    mut skill_counts: ResMut<InGameSkillCounts>,
) {
	let Some(window) = windows.iter().next() else { return };
    let Some(level) = game.level_named(&level_selection.level_name) else { return };

    release_rate.0 = level.globals.release_rate as isize;
    start_countdown.0 = FPS as i32;
    drop_countdown.0 = -1; // Not dropping yet.
    timer.0.set_elapsed(Duration::ZERO);

    skill_counts.0.insert(SkillPanelSelection::Climb, level.globals.skills.climbers as isize);
    skill_counts.0.insert(SkillPanelSelection::Umbrella, level.globals.skills.floaters as isize);
    skill_counts.0.insert(SkillPanelSelection::Explode, level.globals.skills.bombers as isize);
    skill_counts.0.insert(SkillPanelSelection::Block, level.globals.skills.blockers as isize);
    skill_counts.0.insert(SkillPanelSelection::Build, level.globals.skills.builders as isize);
    skill_counts.0.insert(SkillPanelSelection::Bash, level.globals.skills.bashers as isize);
    skill_counts.0.insert(SkillPanelSelection::MineDiagonal, level.globals.skills.miners as isize);
    skill_counts.0.insert(SkillPanelSelection::DigVertical, level.globals.skills.diggers as isize);

    // Scale and bevy-ify the ground's objects.
    let ground = &game.grounds[&(level.globals.normal_graphic_set as i32)];
    pub enum AnimationOrImageHandle {
        Animation(Handle<TextureAtlas>),
        Image(Handle<Image>),
    }
    let mut object_handles = HashMap::<i32, AnimationOrImageHandle>::new();
    for (index, animation) in &ground.object_sprites {
        let frame_count = animation.frames.len();
        let anim_or_image: AnimationOrImageHandle;
        if frame_count == 0 {
            continue;
        } else if frame_count == 1 {
            // Static.
            let image_handle = make_image_from_bitmap(&animation.frames[0], animation.width, animation.height, &mut images, true);
            anim_or_image = AnimationOrImageHandle::Image(image_handle);
        } else {
            // Animation.
            let atlas_handle = make_atlas_from_animation(animation, &mut images, &mut texture_atlases, true);
            anim_or_image = AnimationOrImageHandle::Animation(atlas_handle);
        }
        object_handles.insert(index.clone(), anim_or_image);
    }

    // Spawn the level terrain.
    let render = level_renderer::render(level, &game.grounds, &game.specials, false);
    let game_origin_offset_y: f32 = (render.image.height as f32) * POINT_SIZE / 2.; // Y to use for 0 in game coords.
    let level_offset_y = window.height() / 2. - game_origin_offset_y;
    let scaled = multi_scale(&render.image.bitmap, render.image.width, render.image.height, false);
    let slices_raw = slice(&scaled, render.image.width * SCALE, render.image.height * SCALE, render.size.min_x * SCALE as isize);
    let slices = convert_slices_to_bevy(slices_raw, &mut images);
    commands
        .spawn(SpatialBundle{
            // TODO for the start X, do we need to account for the current screen width?
            transform: Transform::from_xyz(-(level.globals.start_screen_xpos as f32 + ORIGINAL_GAME_W as f32 / 2.) * POINT_SIZE, 
                level_offset_y, 1.),
            ..default()
        }).with_children(|parent| {
            // Terrain slices.
            parent.spawn(SpatialBundle::default()).with_children(|parent| {
                for slice in &slices.slices {
                    parent.spawn(SpriteBundle{
                        transform: Transform{
                            translation: Vec3::new((slice.x as f32 + (slice.width as f32 / 2.)) * TEXTURE_SCALE, 0., 2.),
                            scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
                            ..default()
                        },
                        texture: slice.texture.clone(),
                        ..default()
                    });
                }
            });

            // Spawn level objects.
            for object in &level.objects {
                let z_index: f32 = if object.modifier.is_do_not_overwrite_existing_terrain() { 1. } else { 3. };
                let object_info = &ground.ground.object_info[object.obj_id];
                if let Some(handle) = object_handles.get(&(object.obj_id as i32)) {
                    let transform = Transform{
                        scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
                        translation: Vec3::new((object.x as f32 + object_info.width as f32 / 2.) * POINT_SIZE,
                        game_origin_offset_y - (object.y as f32 + object_info.height as f32 / 2.) * POINT_SIZE, 
                        z_index),
                        ..default()
                    };
                    let object_component = ObjectComponent{
                        info: object_info.clone(),
                    };
                    match handle {
                        AnimationOrImageHandle::Animation(anim) => {
                            parent.spawn(SpriteSheetBundle{
                                sprite: TextureAtlasSprite { index: object_info.start_animation_frame_index as usize % object_info.frame_count as usize, ..default() },
                                texture_atlas: anim.clone(),
                                transform, 
                                ..default()
                            })
                            .insert(object_component);
                        },
                        AnimationOrImageHandle::Image(image) => {
                            parent.spawn(SpriteBundle{
                                texture: image.clone(),
                                transform, 
                                ..default()
                            })
                            .insert(object_component);
                        },
                    }
                }
            }

            // Spawn lemmings container.
            lemmings_container_id.0 = parent.spawn(SpatialBundle {
                transform: Transform::from_xyz(0., 0., 4.),
                ..default() 
            }).id();
        })
        .insert(InGameComponent)
        .insert(MapContainerComponent{
            min_x: -render.size.max_x as f32 * POINT_SIZE,
            max_x: -render.size.min_x as f32 * POINT_SIZE,
        });

    // Keep the slices around.
    slices_resource.0 = Some(slices);
}

struct UpdatePanelDigitsEvent; // No params, update them all.
fn update_panel_digits_system(
    mut events: EventReader<UpdatePanelDigitsEvent>,
    level_selection: Res<LevelSelectionResource>,
	game: Res<Game>,
    game_textures: Res<GameTextures>,
    panel_digits: Res<InGamePanelDigits>,
    release_rate: Res<InGameReleaseRate>,
    skill_counts: Res<InGameSkillCounts>,
    mut digits_query: Query<&mut Handle<Image>, With<InGamePanelDigitComponent>>,
) {
    let Some(_ev) = events.iter().next() else { return }; // Quit early if no event.
    let Some(level) = game.level_named(&level_selection.level_name) else { return };
    for (index, pair) in panel_digits.0.iter().enumerate() {
        let Some(button) = SkillPanelSelection::from_index(index as isize) else { continue };
        let value: isize = match button {
            SkillPanelSelection::SpeedMinus => level.globals.release_rate as isize,
            SkillPanelSelection::SpeedPlus => release_rate.0 as isize,
            _ => *skill_counts.0.get(&button).unwrap_or(&0),
        };
        if let Ok(mut handle) = digits_query.get_mut(pair.left) {
            let left_value = value / 10;
            if left_value == 0 {
                *handle = Handle::default(); // No leading zeros.
            } else {
                *handle = game_textures.skill_number_digits.image(true, left_value);
            }
        }
        if let Ok(mut handle) = digits_query.get_mut(pair.right) {
            if value == 0 {
                *handle = Handle::default(); // Show nothing at all if they're both 0.
            } else {
                *handle = game_textures.skill_number_digits.image(false, value % 10);
            }
        }
    }
}

fn enter_and_spawn_bottom_skill_panel(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    windows: Query<&Window>,
    mut bottom_panel_id: ResMut<InGameBottomPanelId>,
    mut skill_selection_indicator_id: ResMut<InGameSkillSelectionIndicatorId>,
    mut speed_selection_indicator_id: ResMut<InGameSpeedSelectionIndicatorId>,
    mut pause_selection_indicator_id: ResMut<InGamePauseSelectionIndicatorId>,
    mut nuke_selection_indicator_id: ResMut<InGameNukeSelectionIndicatorId>,
    mut panel_digits: ResMut<InGamePanelDigits>,
    mut update_panel_digits_events: EventWriter<UpdatePanelDigitsEvent>,
) {
    let Some(window) = windows.iter().next() else { return };

    // Skill panel.
    bottom_panel_id.0 = commands
        .spawn(SpriteBundle{
            sprite: Sprite { anchor: Anchor::BottomCenter, ..default() },
            texture: game_textures.skill_panel.clone(),
            transform: Transform{
                translation: Vec3::new(0., -window.height() / 2., 10.),
                scale: Vec3::new(TEXTURE_SCALE, TEXTURE_SCALE, 1.),
                ..default()
            },        
            ..default()
        })
        .with_children(|parent| {
            skill_selection_indicator_id.0 = parent.spawn(SpriteBundle{
                texture: game_textures.skill_selection.clone(),
                sprite: Sprite { anchor: Anchor::BottomCenter, ..default() },
                transform: Transform{
                    translation: Vec3::new(-99999. * POINT_SIZE / TEXTURE_SCALE, 0., 11.), // Just on top of skill panel.
                    ..default()
                },
                ..default()
            }).insert(InGameSkillSelectionIndicatorComponent)
            .id();

            speed_selection_indicator_id.0 = parent.spawn(SpriteBundle{
                texture: game_textures.speed_selection.clone(),
                sprite: Sprite { anchor: Anchor::BottomCenter, ..default() },
                transform: Transform{
                    translation: Vec3::new(-99999. * POINT_SIZE / TEXTURE_SCALE, 0., 11.),
                    ..default()
                },
                ..default()
            }).insert(InGameSpeedSelectionIndicatorComponent)
            .id();

            pause_selection_indicator_id.0 = parent.spawn(SpriteBundle{
                texture: game_textures.pause_selection.clone(),
                sprite: Sprite { anchor: Anchor::BottomCenter, ..default() },
                transform: Transform{
                    translation: Vec3::new(-99999. * POINT_SIZE / TEXTURE_SCALE, 0., 11.),
                    ..default()
                },
                ..default()
            }).insert(InGamePauseSelectionIndicatorComponent)
            .id();

            nuke_selection_indicator_id.0 = parent.spawn(SpriteBundle{
                texture: game_textures.nuke_selection.clone(),
                sprite: Sprite { anchor: Anchor::BottomCenter, ..default() },
                transform: Transform{
                    translation: Vec3::new(-99999. * POINT_SIZE / TEXTURE_SCALE, 0., 11.),
                    ..default()
                },
                ..default()
            }).insert(InGameNukeSelectionIndicatorComponent)
            .id();

            // Make a container for all the digits.
            parent.spawn(SpatialBundle{
                ..default()
            }).with_children(|parent| {
                let point = POINT_SIZE / TEXTURE_SCALE; // Makes it simpler below.
                let y = 18.5 * point;
                let leftmost_skill: f32 = -9. * sizes::SKILL_PANEL_BUTTON_WIDTH as f32 - 8.;
                let mut ids: Vec<LeftRightEntityPair> = Vec::new();
                for button_index in 0..10 {
                    let x = (leftmost_skill + (button_index as f32 * sizes::SKILL_PANEL_BUTTON_WIDTH as f32)) * point;
                    let transform = Transform{
                        translation: Vec3::new(x, y, 12.),
                        ..default()
                    }; 
                    let bundle = SpriteBundle{transform, ..default()};
                    let left = parent.spawn(bundle.clone()).insert(InGamePanelDigitComponent).id();
                    let right = parent.spawn(bundle).insert(InGamePanelDigitComponent).id();
                    ids.push(LeftRightEntityPair { left, right });
                }
                panel_digits.0 = ids;
            });
        })
        .insert(InGameComponent)
        .insert(InGameBottomPanelComponent)
        .id();

    // Set the digits.
    update_panel_digits_events.send(UpdatePanelDigitsEvent);
}

struct LeftRightEntityPair {
    left: Entity,
    right: Entity,
}

fn update_lemmings(
    mut query: Query<(
        &mut Transform,
        &mut TextureAtlasSprite,
        &mut Handle<TextureAtlas>,
        &mut LemmingComponent,
    )>,
    timer: Res<GameTimer>,
    slices: Res<InGameSlices>,
    game_textures: Res<GameTextures>,
    is_paused: Res<InGameIsPaused>,
) {
    if is_paused.0 { return }
    if !timer.0.just_finished() { return }

    for (mut t, mut tas, mut ta, mut l) in query.iter_mut() {
        // let mut t: Mut<Transform> = t; // Uncomment these for IDE suggestions, but recomment them to remove compile warnings.
        // let mut tas: Mut<TextureAtlasSprite> = tas;
        // let mut l: Mut<LemmingComponent> = l;
        
        let (game_x, game_y) = game_xy_from_translation(&t.translation);
        let bottom_y = game_y + LEMMING_NOMINAL_HEIGHT_HALF;
        let mut texture_frame_count: Option<usize> = None; // Set if you want it to animate, don't set if you jump to 0 on new anim.
        // Check if there's any ground under this lemming.
        let is_ground_under = is_there_ground_at_xy(game_x, bottom_y, slices.0.as_ref());
        if let Some(skill) = l.skill_in_use {
            match skill {
                SkillPanelSelection::DigVertical => {
                    if ta.id() != game_textures.digging.id() {
                        *ta = game_textures.digging.clone();
                        tas.index = 0;
                    } else {
                        texture_frame_count = Some(game_textures.digging_count);
                    }
                },
                _ => todo!(),
            }
        } else {
            // No skill being used, just walking.
            if is_ground_under {
                let facing_direction_x_delta: i32 = if l.is_facing_right { 1 } else { -1 };
                let game_x_in_direction = game_x + facing_direction_x_delta;
                // These keep track of 'is there ground where i'm walking'.
                // TODO optimise away the fact that these all use the same x, thus same slice.
                let is_ground_3down = is_there_ground_at_xy(game_x_in_direction, bottom_y + 3, slices.0.as_ref());
                let is_ground_2down = is_there_ground_at_xy(game_x_in_direction, bottom_y + 2, slices.0.as_ref());
                let is_ground_1down = is_there_ground_at_xy(game_x_in_direction, bottom_y + 1, slices.0.as_ref());
                let is_ground_on_same_level = is_there_ground_at_xy(game_x_in_direction, bottom_y, slices.0.as_ref());
                let is_ground_1up = is_there_ground_at_xy(game_x_in_direction, bottom_y - 1, slices.0.as_ref());
                let is_ground_2up = is_there_ground_at_xy(game_x_in_direction, bottom_y - 2, slices.0.as_ref());
                let is_ground_3up = is_there_ground_at_xy(game_x_in_direction, bottom_y - 3, slices.0.as_ref()); // Jump.
                let is_ground_4up = is_there_ground_at_xy(game_x_in_direction, bottom_y - 4, slices.0.as_ref());
                let is_ground_5up = is_there_ground_at_xy(game_x_in_direction, bottom_y - 5, slices.0.as_ref());
                let is_ground_6up = is_there_ground_at_xy(game_x_in_direction, bottom_y - 6, slices.0.as_ref());
                let is_ground_7up = is_there_ground_at_xy(game_x_in_direction, bottom_y - 7, slices.0.as_ref()); // Blocked.
                let is_ground_8up = is_there_ground_at_xy(game_x_in_direction, bottom_y - 8, slices.0.as_ref());
                let is_ground_9up = is_there_ground_at_xy(game_x_in_direction, bottom_y - 9, slices.0.as_ref());
                // Jumping is if you walk 3-6 pixels up.
                let is_blocked = is_ground_7up || is_ground_8up || is_ground_9up;
                if is_blocked { // Turn around.
                    l.is_facing_right ^= true; // Toggle.
                    *ta = if l.is_facing_right { game_textures.walking_right.clone() } else { game_textures.walking_left.clone() };
                    tas.index = 0;
                } else { // Not blocked.
                    let should_jump = is_ground_3up || is_ground_4up || is_ground_5up || is_ground_6up;
                    if should_jump { // Take a jump up.
                        let y_offset: f32;
                        if is_ground_6up { y_offset = -6. }
                        else if is_ground_5up { y_offset = -5. }
                        else if is_ground_4up { y_offset = -4. }
                        else { y_offset = -3. }
                        t.translation.x = round_to_nearest_point(t.translation.x + facing_direction_x_delta as f32 * POINT_SIZE);
                        t.translation.y = round_to_nearest_point(t.translation.y - y_offset * POINT_SIZE);
                        *ta = if l.is_facing_right { game_textures.jumping_right.clone() } else { game_textures.jumping_left.clone() };
                        tas.index = 0;
                    } else { // Just walk normally.
                        let y_offset: f32;
                        if is_ground_2up { y_offset = -2. }
                        else if is_ground_1up { y_offset = -1. }
                        else if is_ground_on_same_level { y_offset = 0. }
                        else if is_ground_1down { y_offset = 1. }
                        else if is_ground_2down { y_offset = 2. }
                        else if is_ground_3down { y_offset = 3. }
                        else { y_offset = 1. } // Walking into thin air. Make it drop a little to start with.
                        t.translation.x = round_to_nearest_point(t.translation.x + facing_direction_x_delta as f32 * POINT_SIZE);
                        t.translation.y = round_to_nearest_point(t.translation.y - y_offset * POINT_SIZE);
                        if l.is_facing_right {
                            if ta.id() != game_textures.walking_right.id() {
                                *ta = game_textures.walking_right.clone();
                                tas.index = 0;
                            } else {
                                texture_frame_count = Some(game_textures.walking_right_count);
                            }
                        } else {
                            if ta.id() != game_textures.walking_left.id() {
                                *ta = game_textures.walking_left.clone();
                                tas.index = 0;
                            } else {
                                texture_frame_count = Some(game_textures.walking_left_count);
                            }
                        }
                    }
                }
                //t.translation.x = (t.translation.x + x_delta_from_direction_faced * POINT_SIZE).round_to_nearest_point();
                // Walk left or right. If there's no ground under it to the side, he can climb down or up no dramas without needing to fall.
            } else {
                // TODO if there was nothing under it, iterate DROP_POINTS_PER_FRAME times.
                if l.is_facing_right {
                    if ta.id() != game_textures.falling_right.id() {
                        *ta = game_textures.falling_right.clone();
                        tas.index = 0;
                    } else {
                        texture_frame_count = Some(game_textures.falling_right_count);
                    }
                } else {
                    if ta.id() != game_textures.falling_left.id() {
                        *ta = game_textures.falling_left.clone();
                        tas.index = 0;
                    } else {
                        texture_frame_count = Some(game_textures.falling_left_count);
                    }
                }
                t.translation.y = round_to_nearest_point(t.translation.y - POINT_SIZE); // Round on changes so we don't accumulate some float error.
            }
        }
        if let Some(count) = texture_frame_count {
            tas.index = (tas.index + 1) % count;
        }
    }
}

fn round_to_nearest_point(a: f32) -> f32 {
    (a / POINT_SIZE).round() * POINT_SIZE
}

fn game_xy_from_translation(translation: &Vec3) -> (i32, i32) {
    // Translation 0 means middle, and + numbers go towards the top of the screen.
    let game_y = level_renderer::LEVEL_HEIGHT as f32 / 2. - translation.y / POINT_SIZE;
    let game_x = translation.x / POINT_SIZE;
    (game_x.round() as i32, game_y.round() as i32)
}

// xy are game points, eg y=0=top.
fn is_there_ground_at_xy(x: i32, y: i32, slices_o: Option<&Slices>) -> bool {
    if let Some(slices) = slices_o {
        if let Some(index) = slices.x_to_slice_index_lookup.get(&x) {
            let slice = &slices.slices[*index as usize];
            let game_points_x_offset = x as isize - slice.game_points_x;
            let scaled_x_offset = game_points_x_offset as usize * SCALE;
            let scaled_y_offset = y as usize * SCALE;
            
            // Check all scaled pixels until ground is found.
            // Exit early if ground is found. This is an optimisation because lemmings will more
            // often than not be on the ground.
            for scale_search_y in 0..SCALE {
                let offset = (scaled_y_offset + scale_search_y) * slice.width + scaled_x_offset;
                for scale_search_x in 0..SCALE {
                    if let Some(rgba) = slice.bitmap.get(offset + scale_search_x) {
                        let a = *rgba as u8;
                        if a > 0 {
                            return true
                        }
                    }
                }
            }
            return false
        } else {
            false
        }
    } else {
        false
    }
}
