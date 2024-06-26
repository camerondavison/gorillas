#![allow(clippy::type_complexity)]

use iyes_perf_ui::prelude::*;
use std::cmp;
use std::f32::consts::PI;
use std::time::Duration;

use rand::seq::SliceRandom;
use rand::{thread_rng, RngCore};

use crate::arrow;
use crate::physics::PhysicsPlugin;
use crate::players::PlayersPlugin;
use crate::prelude::*;
use crate::wind::WindPlugin;

#[derive(Component)]
pub(crate) struct BuildingBrick;

#[derive(Component)]
struct LeftBoard;

#[derive(Debug, States, Clone, Hash, Default, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum Action {
    #[default]
    Enter,
    Throwing,
    Watching,
    Winner,
}

#[derive(Component)]
pub(crate) struct ThrowIndicator;

#[derive(Component)]
pub(crate) struct AngleSpeed {
    pub(crate) angle: u8,
    pub(crate) speed: u8,
}

impl Default for AngleSpeed {
    fn default() -> Self {
        AngleSpeed {
            angle: 45,
            speed: 30,
        }
    }
}

#[derive(Component)]
pub(crate) struct Explosion;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum InGameplaySet {
    Watchers,
    Gorillas,
    Collisions,
    TurnChanges,
    Movement,
}

pub(crate) struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // default
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Gorillas".to_string(),
                resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(FIXED_HZ)) // my monitor only does this
        // debug
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
        // objects and colors
        .insert_resource(ClearColor(Color::rgb_u8(126, 161, 219)))
        .add_plugins(ShapePlugin)
        // our plugins
        .add_plugins(audio::GorillasAudioPlugin)
        .add_plugins(CollisionPlugin)
        .add_plugins(PlayersPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(WindPlugin)
        .add_plugins(BananaPlugin)
        .init_state::<Action>()
        // Startup
        .add_systems(Startup, (setup, setup_arena))
        // set ordering
        .configure_sets(
            Update,
            (
                InGameplaySet::Gorillas.run_if(in_state(Action::Enter)),
                InGameplaySet::TurnChanges.after(InGameplaySet::Collisions),
            ),
        )
        // Update
        .add_systems(
            Update,
            (
                (state_logger, update_text_left).in_set(InGameplaySet::Watchers),
                (throw_indicator, rotate_and_change_velocity_input).in_set(InGameplaySet::Gorillas),
                (
                    next_player_system.run_if(in_state(Action::Watching)),
                    winner_player_system,
                )
                    .in_set(InGameplaySet::TurnChanges),
            ),
        )
        .add_systems(OnEnter(Action::Enter), spawn_throw_indicator)
        .add_systems(OnExit(Action::Throwing), cleanup_system::<ThrowIndicator>)
        .add_systems(OnEnter(Action::Winner), cleanup_system::<ThrowIndicator>)
        .add_systems(Update, bevy::window::close_on_esc);
    }
}

fn cleanup_system<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Cameras
    commands.spawn(Camera2dBundle::default());

    // Debug
    // commands.spawn(PerfUiCompleteBundle::default());

    // Text
    let font_bold = asset_server.load("fonts/FiraSans-Bold.ttf");
    let font_medium = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.spawn((
        LeftBoard,
        TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Turn: ".to_string(),
                        style: TextStyle {
                            font: font_bold.clone(),
                            font_size: 30.0,
                            color: Color::BLACK,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: font_medium.clone(),
                            font_size: 30.0,
                            color: Color::BLACK,
                        },
                    },
                    TextSection {
                        value: "\nAction: ".to_string(),
                        style: TextStyle {
                            font: font_bold.clone(),
                            font_size: 30.0,
                            color: Color::BLACK,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: font_medium.clone(),
                            font_size: 30.0,
                            color: Color::BLACK,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: font_bold,
                            font_size: 30.0,
                            color: Color::BLACK,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: font_medium,
                            font_size: 30.0,
                            color: Color::BLACK,
                        },
                    },
                ],
                ..default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        },
    ));
}

fn setup_arena(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Random
    let mut rng = thread_rng();

    // Buildings
    let colors = vec![
        Color::rgb_u8(174, 177, 166),
        Color::rgb_u8(98, 88, 81),
        Color::rgb_u8(208, 208, 181),
    ];
    let num_buildings = (SCREEN_WIDTH / BUILDING_WIDTH).round() as i8;
    assert_eq!(num_buildings as f32 * BUILDING_WIDTH, SCREEN_WIDTH);

    let start_left = -SCREEN_WIDTH / 2.0;
    let start_bottom = -SCREEN_HEIGHT / 2.0;
    for i in 0..num_buildings {
        let n = i as f32;
        let height = rng.next_u32() as f32 % (SCREEN_HEIGHT / 2.0) + SCREEN_HEIGHT / 8.0;
        let height = f32::round(height / BUILDING_BRICK_HEIGHT) * BUILDING_BRICK_HEIGHT;
        let color = *colors.choose(&mut rng).unwrap_or(&Color::BLACK);
        let x = start_left + BUILDING_WIDTH / 2.0 + (BUILDING_WIDTH * n);
        spawn_building(
            format!("b{}", i),
            &mut commands,
            color,
            BUILDING_WIDTH,
            height,
            x,
            start_bottom + (height / 2.0),
        );

        if i == 0 || i == num_buildings - 1 {
            let g = if i == 0 {
                Gorilla::one("Player 1".to_string())
            } else {
                Gorilla::two("Player 2".to_string())
            };

            let gorilla_y = start_bottom + height + GORILLA_HEIGHT / 2.0;
            commands.spawn((
                g,
                SpriteBundle {
                    transform: Transform {
                        translation: Vec2::new(x, gorilla_y).extend(GORILLA_Z_INDEX),
                        scale: Vec2::new(GORILLA_WIDTH, GORILLA_HEIGHT).extend(1.0),
                        ..default()
                    },
                    texture: asset_server.load("sprites/gorilla.png"),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(1.0, 1.0)),
                        ..default()
                    },
                    ..default()
                },
                AngleSpeed::default(),
                Collider,
            ));
        }
    }
}

fn state_logger(mut action_change: EventReader<StateTransitionEvent<Action>>) {
    for chg in action_change.read() {
        info!("Saw state change from {:?} to {:?}", chg.before, chg.after);
    }
}

fn spawn_building(
    name: String,
    commands: &mut Commands,
    color: Color,
    width: f32,
    height: f32,
    x: f32,
    y: f32,
) {
    let num_bricks_width = f32::round(width / BUILDING_BRICK_WIDTH) as usize;
    let num_bricks_height = f32::round(height / BUILDING_BRICK_HEIGHT) as usize;
    debug!("spawning [{name}] ... {width}x{height} bricks {num_bricks_width}x{num_bricks_height} @ center={x},{y}");
    for r in 0..num_bricks_height {
        for c in 0..num_bricks_width {
            let bx = x - (width / 2.0)
                + (BUILDING_BRICK_WIDTH / 2.0)
                + (c as f32 * BUILDING_BRICK_WIDTH);
            let by = y - (height / 2.0)
                + (BUILDING_BRICK_HEIGHT / 2.0)
                + (r as f32 * BUILDING_BRICK_HEIGHT);
            debug!("spawning brick for [{name}] ... center={bx},{by}");
            commands.spawn((
                BuildingBrick,
                SpriteBundle {
                    transform: Transform {
                        translation: Vec2::new(bx, by).extend(BUILDING_Z_INDEX),
                        scale: Vec2::new(BUILDING_BRICK_WIDTH, BUILDING_BRICK_HEIGHT).extend(1.0), // scale z=1.0 in 2D
                        ..default()
                    },
                    sprite: Sprite { color, ..default() },
                    ..default()
                },
                Collider,
            ));
        }
    }
}

fn winner_player_system(
    mut next_action: ResMut<NextState<Action>>,
    mut next_player: ResMut<NextState<Player>>,
    mut gorilla_collision_event: EventReader<GorillaCollisionEvent>,
) {
    for event in gorilla_collision_event.read() {
        info!("saw gorilla collision on {:?}", &event.player);
        // set the winner to the other player
        next_player.set(match event.player {
            Player::One => Player::Two,
            Player::Two => Player::One,
        });
        next_action.set(Action::Winner);
    }
}

fn next_player_system(
    action: Res<State<Action>>,
    mut next_action: ResMut<NextState<Action>>,
    player: Res<State<Player>>,
    mut next_player: ResMut<NextState<Player>>,
    banana_collision_event: EventReader<BananaCollisionEvent>,
    banana_gone_event: EventReader<BananaGoneEvent>,
) {
    if !banana_collision_event.is_empty() || !banana_gone_event.is_empty() {
        info!(
            "next player, current is {:?}, action is {:?}",
            player, action
        );
        next_player.set(match player.get() {
            Player::One => Player::Two,
            Player::Two => Player::One,
        });
        next_action.set(Action::Enter);
    }
}

fn spawn_throw_indicator(mut commands: Commands, mut next_action: ResMut<NextState<Action>>) {
    // spawn throw indicator
    info!("spawn throw indicator");
    let outline_color: Color = *Color::ORANGE_RED.clone().set_a(0.5);
    let fill_color: Color = *Color::ORANGE.clone().set_a(0.5);
    commands.spawn((
        ThrowIndicator,
        arrow::build_arrow_shape(
            outline_color,
            fill_color,
            60,
            30,
            0.0,
            0.0,
            THROW_IND_Z_INDEX,
        ),
    ));
    next_action.set(Action::Enter);
}

fn throw_indicator(
    mut throw_indicator_query: Query<&mut Transform, (With<ThrowIndicator>, Without<Gorilla>)>,
    gorilla_query: Query<(&Gorilla, &AngleSpeed, &Transform)>,
    action: Res<State<Action>>,
    player: Res<State<Player>>,
) {
    if let Ok(ref mut thrown_transform) = throw_indicator_query.get_single_mut() {
        if action.get() == &Action::Enter {
            for (gorilla, gorilla_as, gorilla_transform) in gorilla_query.iter() {
                if player.get() == &gorilla.player {
                    thrown_transform.translation = Vec3::new(
                        gorilla_transform.translation.x,
                        gorilla_transform.translation.y,
                        THROW_IND_Z_INDEX,
                    );
                    let angle = gorilla_as.angle as f32;
                    thrown_transform.rotation = match player.get() {
                        // player1 is 180 == PI rotations, 90 == PI/2.0, 0 == 0
                        Player::One => Quat::from_rotation_z(angle / 180.0 * PI),
                        // player1 is 0 == PI rotations, 90 == PI/2.0, 180 == 0
                        Player::Two => Quat::from_rotation_z((180.0 - angle) / 180.0 * PI),
                    };
                    // 30 == 1.0 length, 60 == 2.0 length
                    let speed = gorilla_as.speed as f32;
                    thrown_transform.scale.x = speed / 30.0;
                }
            }
        }
    }
}

struct MoveArrowState {
    timer: Timer,
}
impl Default for MoveArrowState {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(150), TimerMode::Once),
        }
    }
}

fn rotate_and_change_velocity_input(
    time: Res<Time>,
    player: Res<State<Player>>,
    mut query_angle_speed: Query<(&Gorilla, &mut AngleSpeed)>,
    mut move_arrow_state: Local<MoveArrowState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for (ref mut g, ref mut a) in query_angle_speed.iter_mut() {
        if player.get() == &g.player {
            if keyboard_input.any_just_pressed([
                KeyCode::ArrowUp,
                KeyCode::ArrowDown,
                KeyCode::ArrowLeft,
                KeyCode::ArrowRight,
            ]) {
                move_arrow_state.timer.reset();
                mutate_speed_angle(&keyboard_input, a);
            }
            if keyboard_input.any_pressed([
                KeyCode::ArrowUp,
                KeyCode::ArrowDown,
                KeyCode::ArrowLeft,
                KeyCode::ArrowRight,
            ]) {
                move_arrow_state.timer.tick(time.delta());
                if move_arrow_state.timer.finished() {
                    mutate_speed_angle(&keyboard_input, a);
                }
            }
        }
    }
}

fn mutate_speed_angle(keyboard_input: &Res<ButtonInput<KeyCode>>, a: &mut AngleSpeed) {
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        a.angle = cmp::min(230, a.angle + 1);
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        a.angle = cmp::max(0, a.angle as i16 - 1) as u8;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        a.speed = cmp::min(200, a.speed + 1);
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        a.speed = cmp::max(10, a.speed - 1);
    }
}

fn update_text_left(
    action: Res<State<Action>>,
    player: Res<State<Player>>,
    mut query: Query<&mut Text, With<LeftBoard>>,
    name_query: Query<(&Gorilla, &AngleSpeed)>,
) {
    let mut text = query.single_mut();
    if let Some((g, a)) = name_query.iter().find(|(g, _)| &g.player == player.get()) {
        text.sections[1].value = g.name.to_string();

        let (action, v) = match action.get() {
            Action::Enter => (
                "How do you want to throw?",
                ("\nVelocity: ", format!("{}(m/s) @ {}°", a.speed, a.angle)),
            ),
            Action::Throwing => ("Chunk", ("", "".to_string())),
            Action::Watching => ("Whoa!", ("", "".to_string())),
            Action::Winner => ("Winner !!!", ("", "".to_string())),
        };
        text.sections[3].value = action.to_string();
        text.sections[4].value = v.0.to_string();
        text.sections[5].value = v.1;
    } else {
        error!("unable to find gorilla for player {:?}", player.get());
    }
}
