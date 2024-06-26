use std::f32::consts::PI;

use bevy::input::common_conditions::input_just_pressed;

use crate::game::{Action, AngleSpeed};
use crate::prelude::*;

#[derive(Component)]
pub(crate) struct Banana;

pub(crate) struct BananaPlugin;
impl Plugin for BananaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            throw_banana
                .run_if(in_state(Action::Enter))
                .run_if(input_just_pressed(KeyCode::Space)),
            watch_banana.run_if(in_state(Action::Throwing))));
    }
}

fn throw_banana(
    mut next_action: ResMut<NextState<crate::game::Action>>,
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
    mut next_action: ResMut<NextState<Action>>,
    gorilla_query: Query<&Transform, With<Gorilla>>,
    banana_query: Query<&Transform, With<Banana>>,
) {
    if let Ok(bt) = banana_query.get_single() {
            let mut min_distance = f32::MAX;
        // todo: this is a little wrong because it is finding the distance to both gorillas
            for t in gorilla_query.iter() {
                min_distance = min_distance.min(t.translation.distance(bt.translation))
            }
            if min_distance > 50.0 {
                next_action.set(Action::Watching);

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