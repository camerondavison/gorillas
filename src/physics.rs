use crate::constants::TIME_STEP;
use crate::prelude::*;

#[derive(Component, Deref, DerefMut, Debug)]
pub(crate) struct Velocity(pub(crate) Vec2);

#[derive(Component, Deref, DerefMut, Debug)]
pub(crate) struct Acceleration(pub(crate) Vec2);

#[derive(Component, Deref, DerefMut, Debug)]
pub(crate) struct Rotation(pub(crate) Quat);

#[derive(Component)]
pub(crate) struct Gravity;

pub(crate) struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_gravity)
            .add_systems(Update, (apply_acceleration, apply_velocity, apply_rotation));
    }
}

fn setup_gravity(mut commands: Commands) {
    commands.spawn((Gravity, Acceleration(Vec2::new(0.0, GRAVITY_Y_ACCEL))));
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

fn apply_rotation(mut query: Query<(&mut Transform, &Rotation)>) {
    for (mut transform, rotation) in query.iter_mut() {
        transform.rotation *= rotation.0;
    }
}
