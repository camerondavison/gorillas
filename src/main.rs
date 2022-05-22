use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::{Rng, RngCore, thread_rng};
use rand::seq::SliceRandom;

const BUILDING_WIDTH: f32 = 160.0;
const SCREEN_WIDTH: f32 = 1280.0;
const SCREEN_HEIGHT: f32 = 720.0;

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
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

#[derive(Component)]
struct Building;

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
            start_left + (BUILDING_WIDTH * n),
            start_bottom,
        );
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
        sprite: Sprite { color, anchor: Anchor::BottomLeft, ..default() },
        ..default()
    });
}
