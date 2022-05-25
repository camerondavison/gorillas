use bevy::core::FixedTimestep;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy_kira_audio::{Audio, AudioPlugin, AudioSource};
use bevy_prototype_lyon::prelude::*;
use rand::seq::SliceRandom;
use rand::{thread_rng, RngCore};
use std::f32::consts::PI;

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;
const PIXEL_STEP_SIZE: f32 = 10.0;

// Define sizes
const BUILDING_WIDTH: f32 = 160.0;
const SCREEN_WIDTH: f32 = 1280.0;
const SCREEN_HEIGHT: f32 = 720.0;
const BANANA_WIDTH: f32 = 20.0;
const BANANA_HEIGHT: f32 = 20.0;
const GORILLA_HEIGHT: f32 = 64.0;
const GORILLA_WIDTH: f32 = 32.0;
const EXPLOSION_START_RADIUS: f32 = BANANA_WIDTH / 2.0;

// Speeds
const GRAVITY_Y_ACCEL: f32 = -9.8 * PIXEL_STEP_SIZE;

// Z index
const BUILDING_Z_INDEX: f32 = 1.0;
const BANANA_Z_INDEX: f32 = 3.0;
const GORILLA_Z_INDEX: f32 = 2.0;
const EXPLOSION_Z_INDEX: f32 = 5.0;

#[derive(Component)]
struct Building;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Banana;

#[derive(Component)]
struct LeftBoard;

#[derive(Component)]
struct RightBoard;

#[derive(Component, Deref, DerefMut, Debug)]
struct Velocity(Vec2);

#[derive(Component)]
struct Wind;

#[derive(Component)]
struct Gravity;

#[derive(Component, Deref)]
struct Acceleration(Vec2);

#[derive(Default)]
struct CollisionEvent;

#[derive(Component)]
struct CurrentPlayersTurnText;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
enum Player {
    One,
    Two,
}
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
enum Action {
    Enter,
    Throwing,
    Watching,
    Winner,
}
struct GameState {
    player: Player,
    action: Action,
}
impl Default for GameState {
    fn default() -> Self {
        Self {
            player: Player::One,
            action: Action::Enter,
        }
    }
}

#[derive(Component)]
struct Gorilla(Player);

#[derive(Component)]
struct AngleSpeed {
    angle: u8,
    speed: u8,
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
struct Name(String);

struct ExplosionSound(Handle<AudioSource>);

#[derive(Component)]
struct Explosion;

fn main() {
    let background_color: Color = Color::rgb_u8(126, 161, 219); //cornflower blue

    App::new()
        .insert_resource(ClearColor(background_color))
        .insert_resource(WindowDescriptor {
            title: "Gorillas".to_string(),
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            resizable: false,
            ..Default::default()
        })
        .init_resource::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(ShapePlugin)
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .add_system(change_action)
        .add_system(throw_banana)
        .add_system(watch_banana)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(check_for_collisions)
                .with_system(apply_acceleration.before(check_for_collisions))
                .with_system(apply_velocity.before(check_for_collisions))
                .with_system(play_collision_sound.after(check_for_collisions))
                .with_system(animate_explosion.after(check_for_collisions)),
        )
        .add_system(update_text_left)
        .add_system(update_text_right)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Random
    let mut rng = thread_rng();

    // Cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    // sounds
    let explosion_sounds = asset_server.load("sounds/explosion.mp3");
    commands.insert_resource(ExplosionSound(explosion_sounds));

    // Text
    let font_bold = asset_server.load("fonts/FiraSans-Bold.ttf");
    let font_medium = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands
        .spawn()
        .insert(LeftBoard)
        .insert_bundle(TextBundle {
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
                ],
                ..default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..default()
                },
                ..default()
            },
            ..default()
        });

    commands
        .spawn()
        .insert(RightBoard)
        .insert_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: font_medium,
                            font_size: 30.0,
                            color: Color::BLACK,
                        },
                    },
                    TextSection {
                        value: " <- Banana".to_string(),
                        style: TextStyle {
                            font: font_bold,
                            font_size: 30.0,
                            color: Color::BLACK,
                        },
                    },
                ],
                ..default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    right: Val::Px(100.0),
                    ..default()
                },
                ..default()
            },
            ..default()
        });

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
        let color = *colors.choose(&mut rng).unwrap_or(&Color::BLACK);
        let x = start_left + BUILDING_WIDTH / 2.0 + (BUILDING_WIDTH * n);
        spawn_building(
            commands.spawn(),
            color,
            BUILDING_WIDTH,
            height,
            x,
            start_bottom + (height / 2.0),
        );

        let gorilla_color = Color::DARK_GREEN;
        if i == 0 || i == num_buildings - 1 {
            let (c, n) = if i == 0 {
                (Gorilla(Player::One), "Player 1")
            } else {
                (Gorilla(Player::Two), "Player 2")
            };

            let gorilla_y = start_bottom + height + GORILLA_HEIGHT / 2.0;
            commands
                .spawn()
                .insert(c)
                .insert_bundle(SpriteBundle {
                    transform: Transform {
                        translation: Vec2::new(x, gorilla_y).extend(GORILLA_Z_INDEX),
                        scale: Vec2::new(GORILLA_WIDTH, GORILLA_HEIGHT).extend(1.0), // scale z=1.0 in 2D
                        ..default()
                    },
                    sprite: Sprite {
                        color: gorilla_color,
                        ..default()
                    },
                    ..default()
                })
                .insert(Name(n.to_string()))
                .insert(Collider)
                .insert(AngleSpeed::default());
        }
    }

    // World
    commands
        .spawn()
        .insert(Gravity)
        .insert(Acceleration(Vec2::new(0.0, GRAVITY_Y_ACCEL)));
    // todo: wind
    // commands.spawn().insert(Wind).insert(Acceleration(Vec2::new(10.0, 0.0)));
    // use bevy_prototype_lyon maybe? to draw wind using svg.
}

fn apply_acceleration(
    acceleration_query: Query<&Acceleration>,
    mut velocity_query: Query<&mut Velocity>,
) {
    for acc in acceleration_query.iter() {
        for mut velocity in velocity_query.iter_mut() {
            velocity.x += acc.x * TIME_STEP;
            velocity.y += acc.y * TIME_STEP;
        }
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}

fn spawn_building(
    mut commands: EntityCommands,
    color: Color,
    width: f32,
    height: f32,
    x: f32,
    y: f32,
) {
    info!("spawning ... {width}x{height} @ center={x},{y}");
    commands
        .insert(Building)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec2::new(x, y).extend(BUILDING_Z_INDEX),
                scale: Vec2::new(width, height).extend(1.0), // scale z=1.0 in 2D
                ..default()
            },
            sprite: Sprite { color, ..default() },
            ..default()
        })
        .insert(Collider);
}

fn check_for_collisions(
    mut commands: Commands,
    banana_query: Query<(Entity, &Transform), With<Banana>>,
    collider_query: Query<(&Transform, Option<&Building>, Option<&Gorilla>), With<Collider>>,
    mut player_turn: ResMut<GameState>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    if let Ok((banana_entity, banana_transform)) = banana_query.get_single() {
        // if off screen
        if banana_transform.translation.x <= -SCREEN_WIDTH / 2.0
            || banana_transform.translation.x >= SCREEN_WIDTH / 2.0
        {
            commands.entity(banana_entity).despawn();
            next_player(&mut player_turn);
        } else {
            for (transform, _maybe_building, maybe_gorilla) in collider_query.iter() {
                if let Some(gorilla) = maybe_gorilla {
                    if gorilla.0 == player_turn.player {
                        continue;
                    }
                }
                let collision = collide(
                    banana_transform.translation,
                    banana_transform.scale.truncate(),
                    transform.translation,
                    transform.scale.truncate(),
                );
                if collision.is_some() {
                    spawn_explosion(banana_transform.translation.truncate(), commands.spawn());
                    collision_events.send_default();
                    commands.entity(banana_entity).despawn();
                    if maybe_gorilla.is_some() {
                        player_turn.action = Action::Winner;
                    } else {
                        next_player(&mut player_turn);
                    }
                }
            }
        }
    }
}

fn spawn_explosion(banana_pos: Vec2, mut commands: EntityCommands) {
    let shape = shapes::RegularPolygon {
        sides: 10,
        feature: shapes::RegularPolygonFeature::Radius(EXPLOSION_START_RADIUS),
        ..shapes::RegularPolygon::default()
    };
    commands
        .insert(Explosion)
        .insert_bundle(GeometryBuilder::build_as(
            &shape,
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::ORANGE_RED),
                outline_mode: StrokeMode::new(Color::BLACK, 2.0),
            },
            Transform {
                translation: banana_pos.extend(EXPLOSION_Z_INDEX),
                ..default()
            },
        ));
}

fn animate_explosion(
    mut commands: Commands,
    mut explosion_query: Query<(Entity, &Explosion, &mut Transform)>,
) {
    for (e, _, ref mut t) in explosion_query.iter_mut() {
        if t.scale.x > 5.0 {
            commands.entity(e).despawn();
        } else {
            t.scale *= 1.0 + 5.0 * TIME_STEP;
        }
    }
}

fn next_player(player_turn: &mut ResMut<GameState>) {
    match player_turn.player {
        Player::One => {
            player_turn.player = Player::Two;
        }
        Player::Two => {
            player_turn.player = Player::One;
        }
    }
    player_turn.action = Action::Enter;
}

fn play_collision_sound(
    mut collision_events: EventReader<CollisionEvent>,
    audio: Res<Audio>,
    sound: Res<ExplosionSound>,
) {
    if collision_events.iter().count() > 0 {
        audio.play(sound.0.clone());
    }
}

fn change_action(
    player_turn: ResMut<GameState>,
    mut query_angle_speed: Query<(&Gorilla, &mut AngleSpeed)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if player_turn.action == Action::Enter {
        for (g, ref mut a) in query_angle_speed.iter_mut() {
            if g.0 == player_turn.player {
                if keyboard_input.just_pressed(KeyCode::Up) {
                    a.angle += 1
                }
                if keyboard_input.just_pressed(KeyCode::Down) {
                    a.angle -= 1
                }
                if keyboard_input.just_pressed(KeyCode::Right) {
                    a.speed += 1
                }
                if keyboard_input.just_pressed(KeyCode::Left) {
                    a.speed -= 1
                }
            }
        }
    }
}

fn throw_banana(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_turn: ResMut<GameState>,
    gorilla_query: Query<(&Gorilla, &Transform, &AngleSpeed)>,
    mut commands: Commands,
) {
    if player_turn.action == Action::Enter && keyboard_input.just_pressed(KeyCode::Space) {
        for (g, t, a) in gorilla_query.iter() {
            if g.0 == player_turn.player {
                let angle = a.angle;
                let speed = a.speed;
                // if left alone compass loos like this, but we want to make 90 straight up
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
                    (radians).sin() * (speed as f32),
                    (radians).cos() * (speed as f32),
                );

                // scale, then reverse for player 2
                v *= PIXEL_STEP_SIZE / 1.5;
                if player_turn.player == Player::Two {
                    v.x *= -1.0
                }

                spawn_banana(commands.spawn(), t.translation.truncate(), t.scale, v);
                player_turn.action = Action::Throwing;
            }
        }
    }
}

fn watch_banana(
    mut player_turn: ResMut<GameState>,
    gorilla_query: Query<&Transform, With<Gorilla>>,
    banana_query: Query<&Transform, With<Banana>>,
) {
    if let Ok(bt) = banana_query.get_single() {
        if player_turn.action == Action::Throwing {
            let mut min_distance = f32::MAX;
            for t in gorilla_query.iter() {
                min_distance = min_distance.min(t.translation.distance(bt.translation))
            }
            if min_distance > 50.0 {
                player_turn.action = Action::Watching
            }
        }
    }
}

fn spawn_banana(mut commands: EntityCommands, g_pos: Vec2, _g_size: Vec3, initial_velocity: Vec2) {
    commands
        .insert(Banana)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: g_pos.extend(BANANA_Z_INDEX),
                scale: Vec2::new(BANANA_WIDTH, BANANA_HEIGHT).extend(1.0), // scale z=1.0 in 2D
                ..default()
            },
            sprite: Sprite {
                color: Color::YELLOW,
                ..default()
            },
            ..default()
        })
        .insert(Velocity(initial_velocity));
}

fn update_text_left(
    player_turn: Res<GameState>,
    mut query: Query<&mut Text, With<LeftBoard>>,
    name_query: Query<(&Gorilla, &AngleSpeed, &Name)>,
) {
    let mut text = query.single_mut();
    if let Some((_, a, n)) = name_query
        .iter()
        .find(|(g, _, _)| g.0 == player_turn.player)
    {
        text.sections[1].value = n.0.to_string();

        let (action, v) = match player_turn.action {
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
        error!("unable to find gorilla for player {:?}", player_turn.player)
    }
}

fn update_text_right(
    player_turn: Res<GameState>,
    mut query: Query<&mut Text, With<RightBoard>>,
    banana_query: Query<&mut Velocity, With<Banana>>,
) {
    let mut text = query.single_mut();
    let v = if let Ok(velocity) = banana_query.get_single() {
        match player_turn.action {
            Action::Enter | Action::Winner => "".to_string(),
            Action::Throwing | Action::Watching => {
                format!("{}x{}", velocity.x.round(), velocity.y.round())
            }
        }
    } else {
        "".to_string()
    };
    text.sections[0].value = v;
}
