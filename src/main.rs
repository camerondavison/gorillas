use bevy::core::FixedTimestep;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use rand::{RngCore, thread_rng};
use rand::seq::SliceRandom;

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;

// Define sizes
const BUILDING_WIDTH: f32 = 160.0;
const SCREEN_WIDTH: f32 = 1280.0;
const SCREEN_HEIGHT: f32 = 720.0;
const BANANA_WIDTH: f32 = 20.0;
const BANANA_HEIGHT: f32 = 20.0;

fn main() {
    let background_color: Color = Color::rgb_u8(126, 161, 219);//cornflower blue

    App::new()
        .insert_resource(ClearColor(background_color))
        .insert_resource(WindowDescriptor {
            title: "Gorillas".to_string(),
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(check_for_collisions)
                .with_system(apply_acceleration.before(check_for_collisions))
                .with_system(apply_velocity.before(check_for_collisions))
                .with_system(play_collision_sound.after(check_for_collisions)),
        )
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

#[derive(Component)]
struct Building;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Banana;

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

fn setup(mut commands: Commands) {
    // Random
    let mut rng = thread_rng();

    // Cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    // Buildings
    let colors = vec![
        Color::rgb_u8(174,177,166),
        Color::rgb_u8(98,88,81),
        Color::rgb_u8(208,208,181),
    ];
    let num_buildings = (SCREEN_WIDTH/BUILDING_WIDTH).round() as i8;
    assert_eq!(num_buildings as f32 * BUILDING_WIDTH, SCREEN_WIDTH);

    let start_left = -SCREEN_WIDTH / 2.0;
    let start_bottom = -SCREEN_HEIGHT / 2.0;
    for i in 0..num_buildings {
        let n = i as f32;
        let height = rng.next_u32() as f32 % (SCREEN_HEIGHT/2.0) + SCREEN_HEIGHT/8.0;
        let color = colors.choose(&mut rng).unwrap_or(&Color::BLACK).clone();
        spawn_building(
            commands.spawn(),
            color,
            BUILDING_WIDTH,
            height,
            start_left + BUILDING_WIDTH / 2.0 + (BUILDING_WIDTH * n),
            start_bottom + (height / 2.0),
        );
    }

    // World
    commands.spawn().insert(Gravity).insert(Acceleration(Vec2::new(0.0,-30.0)));
    commands.spawn().insert(Wind).insert(Acceleration(Vec2::new(10.0, 0.0)));

    // Banana
    commands.spawn().insert(Banana).insert_bundle(SpriteBundle {
        transform: Transform {
            translation: Vec2::new(-SCREEN_WIDTH/2.0+(BUILDING_WIDTH/2.0),SCREEN_HEIGHT/2.0-100.0).extend(0.0),
            scale: Vec2::new(BANANA_WIDTH, BANANA_HEIGHT).extend(1.0), // scale z=1.0 in 2D
            ..default()
        },
        sprite: Sprite { color: Color::YELLOW, ..default() },
        ..default()
    }).insert(Velocity(Vec2::new(0.0, -90.0)));
}

fn apply_acceleration(
    acceleration_query: Query<&Acceleration>,
    mut velocity_query: Query<&mut Velocity>,
) {
    for acc in acceleration_query.iter() {
        for mut velocity in velocity_query.iter_mut() {
            println!("velocity {:?}", velocity);
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
    commands.insert(Building).insert_bundle(SpriteBundle {
        transform: Transform {
            translation: Vec2::new(x, y).extend(0.0),
            scale: Vec2::new(width, height).extend(1.0), // scale z=1.0 in 2D
            ..default()
        },
        sprite: Sprite { color, ..default() },
        ..default()
    }).insert(Collider);
}

fn check_for_collisions(
    mut commands: Commands,
    banana_query: Query<(Entity, &Transform), With<Banana>>,
    collider_query: Query<(Entity, &Transform, Option<&Building>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    if let Ok((b_entity, b_transform)) = banana_query.get_single() {
        let b_size = b_transform.scale.truncate();

        // check collision with walls
        for (_collided_entity, transform, _maybe_building) in collider_query.iter() {
            let collision = collide(
                b_transform.translation,
                b_size,
                transform.translation,
                transform.scale.truncate(),
            );
            if let Some(_collision) = collision {
                collision_events.send_default();
                commands.entity(b_entity).despawn();
            }
        }
    }
}

fn play_collision_sound(
    mut collision_events: EventReader<CollisionEvent>,
) {
    if collision_events.iter().count() > 0 {
        println!("BOOM!!")
    }
}
