#![allow(clippy::type_complexity)]
use bevy::core::FixedTimestep;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy_kira_audio::{Audio, AudioPlugin, AudioSource};
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use rand::prelude::ThreadRng;
use rand::seq::SliceRandom;
use rand::{thread_rng, RngCore};
use std::cmp;
use std::f32::consts::PI;

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;
const PIXEL_STEP_SIZE: f32 = 10.0;

// Define sizes
const BUILDING_WIDTH: f32 = 160.0;
const BUILDING_BRICK_WIDTH: f32 = 32.0; //20.0
const BUILDING_BRICK_HEIGHT: f32 = 8.0; //5.0
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
const BANANA_Z_INDEX: f32 = 4.0;
const GORILLA_Z_INDEX: f32 = 10.0;
const THROW_IND_Z_INDEX: f32 = 12.0;
const EXPLOSION_Z_INDEX: f32 = 15.0;
const WIND_Z_INDEX: f32 = 20.0;

#[derive(Component)]
struct BuildingBrick;

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
struct WindText;

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
    PreEnter,
    Enter,
    Throwing,
    Watching,
    Winner,
}
struct GameState {
    player: Player,
    action: Action,
    key_pressed_at: f64,
}
impl Default for GameState {
    fn default() -> Self {
        Self {
            player: Player::One,
            action: Action::PreEnter,
            key_pressed_at: 0f64,
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
        .add_startup_system(setup_arena)
        .add_event::<CollisionEvent>()
        .add_system(world_changer)
        .add_system(throw_banana)
        .add_system(watch_banana)
        .add_system(update_text_left)
        .add_system(throw_indicator)
        .add_system(change_action)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(check_for_collisions)
                .with_system(apply_acceleration.before(check_for_collisions))
                .with_system(apply_velocity.before(check_for_collisions))
                .with_system(play_collision_sound.after(check_for_collisions))
                .with_system(animate_explosion.after(check_for_collisions)),
        )
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..default()
                },
                ..default()
            },
            ..default()
        });
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
    spawn_wind_wth_accel(&mut commands, &mut rng, asset_server);
}

fn spawn_wind_wth_accel(
    commands: &mut Commands,
    rng: &mut ThreadRng,
    asset_server: Res<AssetServer>,
) -> i32 {
    let font_medium = asset_server.load("fonts/FiraMono-Medium.ttf");
    let wind = (rng.next_u32() % 40) as i32 - 20;
    let raw_length = wind as i16 * 10i16;
    let top = 50.0;
    let right = 300.0;
    let y = SCREEN_HEIGHT / 2.0 - top;
    let x = SCREEN_WIDTH / 2.0 - right;
    commands
        .spawn()
        .insert(Wind)
        .insert_bundle(build_arrow_shape(
            Color::DARK_GRAY,
            Color::GRAY,
            raw_length,
            30,
            x,
            y,
            WIND_Z_INDEX,
        ))
        .insert(Acceleration(Vec2::new(wind as f32, 0.0)));

    //  todo: can we add wind text as a child of something?
    commands.spawn().insert(WindText).insert_bundle(text_bundle(
        font_medium,
        top - 30.0,
        right,
        "wind".to_string(),
    ));
    wind
}

fn text_bundle(font_medium: Handle<Font>, top: f32, right: f32, value: String) -> TextBundle {
    TextBundle {
        text: Text {
            sections: vec![TextSection {
                value,
                style: TextStyle {
                    font: font_medium,
                    font_size: 30.0,
                    color: Color::BLACK,
                },
            }],
            ..default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(top),
                right: Val::Px(right),
                ..default()
            },
            ..default()
        },
        ..default()
    }
}

fn world_changer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    wind_query: Query<Entity, Or<(With<Wind>, With<WindText>)>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::W) {
        for we in wind_query.iter() {
            commands.entity(we).despawn();
        }
        let mut rng = thread_rng();
        let new_wind = spawn_wind_wth_accel(&mut commands, &mut rng, asset_server);
        info!("new wind of {}", new_wind);
    }
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
            commands
                .spawn()
                .insert(BuildingBrick)
                .insert_bundle(SpriteBundle {
                    transform: Transform {
                        translation: Vec2::new(bx, by).extend(BUILDING_Z_INDEX),
                        scale: Vec2::new(BUILDING_BRICK_WIDTH, BUILDING_BRICK_HEIGHT)
                            .extend(1.0), // scale z=1.0 in 2D
                        ..default()
                    },
                    sprite: Sprite { color, ..default() },
                    ..default()
                })
                .insert(Collider);
        }
    }
}

fn check_for_collisions(
    mut commands: Commands,
    banana_query: Query<(Entity, &Transform), With<Banana>>,
    explosion_query: Query<&Transform, With<Explosion>>,
    collider_query: Query<
        (Entity, &Transform, Option<&BuildingBrick>, Option<&Gorilla>),
        With<Collider>,
    >,
    mut player_turn: ResMut<GameState>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    // look up if explosion has hit something
    if let Ok(explosion_transform) = explosion_query.get_single() {
        let mut t = explosion_transform.clone();
        t.scale *= EXPLOSION_START_RADIUS*2.0;

        despawn_from_collision_result(
            format!("explosion"),
            &mut commands,
            &collider_query,
            &mut player_turn,
            &t,
        );
    }

    if player_turn.action != Action::Watching {
        return;
    }

    // look up if our banana has hit something
    if let Ok((banana_entity, banana_transform)) = banana_query.get_single() {
        // if off screen
        if banana_transform.translation.x <= -SCREEN_WIDTH / 2.0
            || banana_transform.translation.x >= SCREEN_WIDTH / 2.0
        {
            commands.entity(banana_entity).despawn();
            next_player(&mut player_turn);
        } else if despawn_from_collision_result(
            format!("banana"),
            &mut commands,
            &collider_query,
            &mut player_turn,
            banana_transform,
        ) {
            spawn_explosion(banana_transform.translation.truncate(), &mut commands);
            collision_events.send_default();
            commands.entity(banana_entity).despawn();
            next_player(&mut player_turn);
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
    player_turn: &mut ResMut<GameState>,
    moving_transform: &Transform,
) -> bool {
    let mut did_collide = false;

    for (e, transform, maybe_building, maybe_gorilla) in collider_query.iter() {
        let collision = collide(
            moving_transform.translation,
            moving_transform.scale.truncate(),
            transform.translation,
            transform.scale.truncate(),
        );
        if collision.is_some() {
            did_collide = true;
            debug!("{collision_name} collided");

            if maybe_gorilla.is_some() {
                player_turn.action = Action::Winner;
            }
            if maybe_building.is_some() {
                commands.entity(e).despawn();
            }
        }
    }

    did_collide
}

fn spawn_explosion(banana_pos: Vec2, commands: &mut Commands) {
    let shape = shapes::RegularPolygon {
        sides: 10,
        feature: shapes::RegularPolygonFeature::Radius(EXPLOSION_START_RADIUS),
        ..shapes::RegularPolygon::default()
    };
    commands
        .spawn()
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
    mut explosion_query: Query<(Entity, &mut Transform), With<Explosion>>,
) {
    for (e, ref mut t) in explosion_query.iter_mut() {
        if t.scale.x > 5.0 {
            commands.entity(e).despawn();
        } else {
            t.scale *= 1.0 + 5.0 * TIME_STEP;
        }
    }
}

fn next_player(gs: &mut ResMut<GameState>) {
    if gs.action != Action::Winner {
        info!("next player, current {:?}", gs.player);
        gs.player = match gs.player {
            Player::One => Player::Two,
            Player::Two => Player::One,
        };
        gs.action = Action::PreEnter;
    }
}

fn throw_indicator(
    mut commands: Commands,
    mut player_turn: ResMut<GameState>,
    gorilla_query: Query<(&Gorilla, &AngleSpeed, &Transform)>,
    mut throw_indicator_query: Query<&mut Transform, (With<ThrowIndicator>, Without<Gorilla>)>,
) {
    if let Ok(ref mut thrown_transform) = throw_indicator_query.get_single_mut() {
        if player_turn.action == Action::Enter {
            for (gorilla, gorilla_as, gorilla_transform) in gorilla_query.iter() {
                if player_turn.player == gorilla.0 {
                    thrown_transform.translation = Vec3::new(
                        gorilla_transform.translation.x,
                        gorilla_transform.translation.y,
                        THROW_IND_Z_INDEX,
                    );
                    let angle = gorilla_as.angle as f32;
                    thrown_transform.rotation = match player_turn.player {
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
    } else if player_turn.action == Action::PreEnter {
        // spawn throw indicator
        info!("spawn throw indicator");
        let outline_color: Color = *Color::ORANGE_RED.clone().set_a(0.5);
        let fill_color: Color = *Color::ORANGE.clone().set_a(0.5);
        commands
            .spawn()
            .insert(ThrowIndicator)
            .insert_bundle(build_arrow_shape(
                outline_color,
                fill_color,
                60,
                30,
                0.0,
                0.0,
                THROW_IND_Z_INDEX,
            ));
        player_turn.action = Action::Enter
    }
}

fn build_arrow_shape(
    outline_color: Color,
    fill_color: Color,
    raw_length: i16,
    height: u16,
    x: f32,
    y: f32,
    z: f32,
) -> ShapeBundle {
    let length = raw_length.abs() as u16;
    let scale = if raw_length < 0 { -1.0 } else { 1.0 };
    let svg_path_string = arrow_path(&length, height);
    GeometryBuilder::build_as(
        &shapes::SvgPathShape {
            svg_doc_size_in_px: Vec2::new(length as f32, height as f32),
            svg_path_string,
        },
        DrawMode::Outlined {
            outline_mode: StrokeMode::color(outline_color),
            fill_mode: FillMode::color(fill_color),
        },
        Transform {
            translation: Vec3::new(x, y, z),
            scale: Vec2::new(scale, 1.0).extend(1.0),
            ..default()
        },
    )
}

fn arrow_path(length: &u16, height: u16) -> String {
    let mut svg_path_string = format!("M {} {}", length / 2, height / 2);
    svg_path_string.push_str(&format!(
        "h {} v -6 l 8 8 l -8 8 v -6 h -{} v -4",
        length, length
    ));
    svg_path_string
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
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut query_angle_speed: Query<(&Gorilla, &mut AngleSpeed)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if game_state.action == Action::Enter {
        for (ref mut g, ref mut a) in query_angle_speed.iter_mut() {
            if g.0 == game_state.player {
                if keyboard_input.any_just_pressed([
                    KeyCode::Up,
                    KeyCode::Down,
                    KeyCode::Left,
                    KeyCode::Right,
                ]) {
                    game_state.key_pressed_at = time.seconds_since_startup();
                    mutate_speed_angle(&keyboard_input, a);
                }
                if keyboard_input.any_pressed([
                    KeyCode::Up,
                    KeyCode::Down,
                    KeyCode::Left,
                    KeyCode::Right,
                ]) && time.seconds_since_startup() - game_state.key_pressed_at > 0.3
                {
                    mutate_speed_angle(&keyboard_input, a);
                }
            }
        }
    }
}

fn mutate_speed_angle(keyboard_input: &Res<Input<KeyCode>>, a: &mut AngleSpeed) {
    if keyboard_input.pressed(KeyCode::Up) {
        a.angle = cmp::min(230, a.angle + 1);
    }
    if keyboard_input.pressed(KeyCode::Down) {
        a.angle = cmp::max(0, a.angle as i16 - 1) as u8;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        a.speed = cmp::min(200, a.speed + 1);
    }
    if keyboard_input.pressed(KeyCode::Left) {
        a.speed = cmp::max(10, a.speed - 1);
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
    mut commands: Commands,
    mut player_turn: ResMut<GameState>,
    gorilla_query: Query<&Transform, With<Gorilla>>,
    banana_query: Query<&Transform, With<Banana>>,
    indicator_query: Query<(Entity, &ThrowIndicator)>,
) {
    if let Ok(bt) = banana_query.get_single() {
        if player_turn.action == Action::Throwing {
            let mut min_distance = f32::MAX;
            for t in gorilla_query.iter() {
                min_distance = min_distance.min(t.translation.distance(bt.translation))
            }
            if min_distance > 50.0 {
                player_turn.action = Action::Watching;
                let (e, _) = indicator_query.single();
                commands.entity(e).despawn();
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
        error!("unable to find gorilla for player {:?}", player_turn.player)
    }
}
