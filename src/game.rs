#![allow(clippy::type_complexity)]
use bevy::math::bounding::{Aabb2d, IntersectsVolume};

use crate::players::PlayersPlugin;
use crate::prelude::*;
use bevy::input::common_conditions::*;

use crate::physics::PhysicsPlugin;
use rand::seq::SliceRandom;
use rand::{thread_rng, RngCore};
use std::cmp;
use std::f32::consts::PI;
use std::time::Duration;
use crate::arrow;
use crate::wind::WindPlugin;

#[derive(Component)]
struct BuildingBrick;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Banana;

#[derive(Component)]
struct LeftBoard;

#[derive(Debug, States, Clone, Hash, Default, Ord, PartialOrd, Eq, PartialEq)]
enum Action {
    #[default]
    PreEnter,
    Enter,
    Throwing,
    Watching,
    Winner,
}

#[derive(Component)]
struct AngleSpeed {
    angle: u8,
    speed: u8,
}

#[derive(Component)]
struct ThrowIndicator;

impl Default for AngleSpeed {
    fn default() -> Self {
        AngleSpeed {
            angle: 45,
            speed: 30,
        }
    }
}

#[derive(Component)]
struct Explosion;

pub(crate) struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Gorillas".to_string(),
                resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::rgb_u8(126, 161, 219)))
        .add_plugins(ShapePlugin)
        // our plugins
        .add_plugins(audio::GorillasAudioPlugin)
        .add_plugins(CollisionPlugin)
        .add_plugins(PlayersPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(WindPlugin)
        .init_state::<Action>()
        // Startup
        .add_systems(Startup, (setup, setup_arena))
        // Update
        .add_systems(
            Update,
            (
                throw_banana
                    .run_if(in_state(Action::Enter))
                    .run_if(input_just_pressed(KeyCode::Space)),
                watch_banana,
                update_text_left,
                throw_indicator,
                state_watcher,
                change_action.run_if(in_state(Action::Enter)),
            ),
        )
        .add_systems(
            Update,
            (
                check_for_collisions_explosion,
                check_for_collisions_banana.run_if(in_state(Action::Watching)),
                animate_explosion,
            )
                .chain(),
        )
        .add_systems(Update, bevy::window::close_on_esc);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Cameras
    commands.spawn(Camera2dBundle::default());

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
                Collider,
                AngleSpeed::default(),
            ));
        }
    }
}

fn state_watcher(mut action_change: EventReader<StateTransitionEvent<Action>>) {
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

fn check_for_collisions_explosion(
    mut commands: Commands,
    explosion_query: Query<&Transform, With<Explosion>>,
    collider_query: Query<
        (Entity, &Transform, Option<&BuildingBrick>, Option<&Gorilla>),
        With<Collider>,
    >,
    action: Res<State<Action>>,
    mut next_action: ResMut<NextState<Action>>,
    player: Res<State<Player>>,
    mut next_player: ResMut<NextState<Player>>,
) {
    // look up if explosion has hit something
    if let Ok(explosion_transform) = explosion_query.get_single() {
        let mut t = explosion_transform.clone();
        t.scale *= EXPLOSION_START_RADIUS * 2.0;

        despawn_from_collision_result(
            format!("explosion"),
            &mut commands,
            &collider_query,
            &t,
            &action,
            &mut next_action,
            &player,
            &mut next_player,
        );
    }
}

fn check_for_collisions_banana(
    mut commands: Commands,
    banana_query: Query<(Entity, &Transform), With<Banana>>,
    collider_query: Query<
        (Entity, &Transform, Option<&BuildingBrick>, Option<&Gorilla>),
        With<Collider>,
    >,
    mut collision_events: EventWriter<CollisionEvent>,
    action: Res<State<Action>>,
    mut next_action: ResMut<NextState<Action>>,
    player: Res<State<Player>>,
    mut next_player: ResMut<NextState<Player>>,
) {
    // look up if our banana has hit something
    if let Ok((banana_entity, banana_transform)) = banana_query.get_single() {
        // if off screen
        if banana_transform.translation.x <= -SCREEN_WIDTH / 2.0
            || banana_transform.translation.x >= SCREEN_WIDTH / 2.0
        {
            commands.entity(banana_entity).despawn();
            next_player_system(
                &action,
                &mut next_action,
                &player,
                &mut next_player,
                Action::PreEnter,
            );
        } else if despawn_from_collision_result(
            format!("banana"),
            &mut commands,
            &collider_query,
            banana_transform,
            &action,
            &mut next_action,
            &player,
            &mut next_player,
        ) {
            spawn_explosion(banana_transform.translation.truncate(), &mut commands);
            collision_events.send_default();
            commands.entity(banana_entity).despawn();
            next_player_system(
                &action,
                &mut next_action,
                &player,
                &mut next_player,
                Action::PreEnter,
            );
        }
    }
}

fn despawn_from_collision_result(
    collision_name: String,
    commands: &mut Commands,
    collider_query: &Query<
        (Entity, &Transform, Option<&BuildingBrick>, Option<&Gorilla>),
        With<Collider>,
    >,
    moving_transform: &Transform,
    action: &Res<State<Action>>,
    mut next_action: &mut ResMut<NextState<Action>>,
    player: &Res<State<Player>>,
    mut next_player: &mut ResMut<NextState<Player>>,
) -> bool {
    let mut did_collide = false;
    let mut did_collide_with_gorilla = false;

    for (e, transform, maybe_building, maybe_gorilla) in collider_query.iter() {
        let collision = Aabb2d::new(
            moving_transform.translation.truncate(),
            moving_transform.scale.truncate() / 2.0,
        )
        .intersects(&Aabb2d::new(
            transform.translation.truncate(),
            transform.scale.truncate() / 2.0,
        ));
        if collision {
            did_collide = true;
            if maybe_gorilla.is_some() {
                did_collide_with_gorilla = true;
                info!("{collision_name} collided with gorilla");
            }
            if maybe_building.is_some() {
                commands.entity(e).despawn();
                info!("{collision_name} collided with building");
            }
        }
    }
    if did_collide_with_gorilla {
        next_player_system(
            &action,
            &mut next_action,
            &player,
            &mut next_player,
            Action::Winner,
        );
    }

    did_collide
}

fn spawn_explosion(banana_pos: Vec2, commands: &mut Commands) {
    let shape = shapes::RegularPolygon {
        sides: 10,
        feature: shapes::RegularPolygonFeature::Radius(EXPLOSION_START_RADIUS),
        ..shapes::RegularPolygon::default()
    };
    commands.spawn((
        Explosion,
        (
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                spatial: SpatialBundle::from_transform(Transform::from_translation(
                    banana_pos.extend(EXPLOSION_Z_INDEX),
                )),
                ..default()
            },
            Fill::color(Color::ORANGE_RED),
            Stroke::new(Color::BLACK, 2.0),
        ),
    ));
}

const EXPLOSION_SIZE: f32 = 3.0;

fn animate_explosion(
    mut commands: Commands,
    mut explosion_query: Query<(Entity, &mut Transform), With<Explosion>>,
) {
    for (e, ref mut t) in explosion_query.iter_mut() {
        if t.scale.x > EXPLOSION_SIZE {
            commands.entity(e).despawn();
        } else {
            t.scale *= 1.0 + 5.0 * TIME_STEP;
        }
    }
}

fn next_player_system(
    action: &Res<State<Action>>,
    next_action: &mut ResMut<NextState<Action>>,
    player: &Res<State<Player>>,
    next_player: &mut ResMut<NextState<Player>>,
    set_next_action: Action,
) {
    if action.get() != &Action::Winner {
        // todo: make if in system?
        info!(
            "next player, current is {:?}, action is {:?}",
            player, action
        );
        next_player.set(match player.get() {
            Player::One => Player::Two,
            Player::Two => Player::One,
        });
        next_action.set(set_next_action);
    }
}

fn throw_indicator(
    mut commands: Commands,
    mut throw_indicator_query: Query<&mut Transform, (With<ThrowIndicator>, Without<Gorilla>)>,
    gorilla_query: Query<(&Gorilla, &AngleSpeed, &Transform)>,
    action: Res<State<Action>>,
    mut next_action: ResMut<NextState<Action>>,
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
    } else if action.get() == &Action::PreEnter {
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

fn change_action(
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

fn throw_banana(
    mut next_action: ResMut<NextState<Action>>,
    player: Res<State<Player>>,
    asset_server: Res<AssetServer>,
    gorilla_query: Query<(&Gorilla, &Transform, &AngleSpeed)>,
    mut commands: Commands,
) {
    for (g, t, a) in gorilla_query.iter() {
        if &g.player == player.get() {
            let angle = a.angle;
            let speed = a.speed;
            // if left alone compass looks like this, but we want to make 90 straight up
            // and for 100 to be behind the head
            //        0
            // 270 <- * -> 90
            //       180
            //
            // so we are just going to do (90 - *degrees*)
            // to make it go
            //       90
            // 180 <- * -> 0
            //       270
            let radians = (90 - angle) as f32 * PI / 180.0;
            let mut v = Vec2::new(
                radians.sin() * (speed as f32),
                radians.cos() * (speed as f32),
            );

            // scale, then reverse for player 2
            v *= PIXEL_STEP_SIZE / 1.5;
            if player.get() == &Player::Two {
                v.x *= -1.0
            }
            spawn_banana(
                &asset_server,
                &mut commands,
                t.translation.truncate(),
                t.scale,
                v,
            );
            next_action.set(Action::Throwing);
        }
    }
}

fn watch_banana(
    mut commands: Commands,
    action: Res<State<Action>>,
    mut next_action: ResMut<NextState<Action>>,
    gorilla_query: Query<&Transform, With<Gorilla>>,
    banana_query: Query<&Transform, With<Banana>>,
    indicator_query: Query<Entity, With<ThrowIndicator>>,
) {
    if let Ok(bt) = banana_query.get_single() {
        if action.get() == &Action::Throwing {
            let mut min_distance = f32::MAX;
            for t in gorilla_query.iter() {
                min_distance = min_distance.min(t.translation.distance(bt.translation))
            }
            if min_distance > 50.0 {
                next_action.set(Action::Watching);
                let e = indicator_query.single();
                commands.entity(e).despawn();
            }
        }
    }
}

fn spawn_banana(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    g_pos: Vec2,
    _g_size: Vec3,
    initial_velocity: Vec2,
) {
    let banana_rotation: Quat = Quat::from_rotation_z(PI * -TIME_STEP);
    commands.spawn((
        Banana,
        SpriteBundle {
            transform: Transform {
                translation: g_pos.extend(BANANA_Z_INDEX),
                scale: Vec2::new(BANANA_WIDTH / 500.0, BANANA_HEIGHT / 500.0).extend(1.0), // scale z=1.0 in 2D
                ..default()
            },
            texture: asset_server.load("sprites/banana.png"),
            ..default()
        },
        Velocity(initial_velocity),
        Rotation(banana_rotation),
    ));
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
            Action::PreEnter | Action::Enter => (
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
